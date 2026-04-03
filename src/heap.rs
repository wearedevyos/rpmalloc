use crate::ffi;
use allocator_api2::alloc::{handle_alloc_error, AllocError, Allocator, Global, Layout};
use allocator_api2::boxed::Box;
use libc::{c_int, size_t};
use std::cmp::max;
use std::ffi::c_void;
use std::marker::PhantomPinned;
use std::pin::Pin;
use std::ptr::{null_mut, NonNull};
use std::slice;

// Taken from std::ptr::Alignment.
const MIN_ALIGN: usize = 1 << 0;

unsafe extern "C" fn euk_alloc_memory_map<A: Allocator>(
    context: *mut c_void,
    size: size_t,
    alignment: size_t,
    _: *mut size_t,
    mapped_size: *mut size_t,
) -> *mut c_void {
    unsafe {
        let interface = &mut *context.cast::<MemoryInterface<A>>();
        mapped_size.write(size);
        interface.mmap(size, alignment)
    }
}

unsafe extern "C" fn euk_alloc_memory_commit<A: Allocator>(
    _: *mut c_void,
    _: *mut c_void,
    _: size_t,
) {
}
unsafe extern "C" fn euk_alloc_memory_decommit<A: Allocator>(
    _: *mut c_void,
    _: *mut c_void,
    _: size_t,
) {
}
unsafe extern "C" fn euk_alloc_map_fail_callback<A: Allocator>(_: *mut c_void, _: size_t) -> c_int {
    return 0;
}
unsafe extern "C" fn euk_alloc_error_callback<A: Allocator>(
    _: *mut c_void,
    _: *const ::std::os::raw::c_char,
) {
    // log::error!("{}", message);
}

unsafe extern "C" fn euk_alloc_memory_unmap<A: Allocator>(
    context: *mut c_void,
    address: *mut c_void,
    _: size_t,
    mapped_size: size_t,
) {
    unsafe {
        let interface = &mut *context.cast::<MemoryInterface<A>>();
        interface.unmap(address, mapped_size)
    }
}

pub struct Heap<A: Allocator = Global> {
    _interface: Pin<Box<MemoryInterface<A>, A>>,
    inner: NonNull<ffi::rpmalloc_heap_t>,
}

impl<A: Allocator + Clone + Default + 'static> Heap<A> {
    pub fn try_new() -> Result<Heap<A>, AllocError> {
        Self::try_new_in(A::default())
    }
}

impl<A: Allocator + Clone + 'static> Heap<A> {
    pub fn try_new_in(alloc: A) -> Result<Heap<A>, AllocError> {
        let mut interface = Box::pin_in(MemoryInterface::new(alloc.clone()), alloc);

        let heap = unsafe {
            let interface = interface.as_mut().get_unchecked_mut();
            (*interface).inner.context = interface as *mut MemoryInterface<A> as *mut c_void;
            ffi::rpmalloc_heap_create(&mut interface.inner)
        };

        Ok(Heap {
            _interface: interface,
            inner: NonNull::new(heap).ok_or(AllocError)?,
        })
    }

    pub unsafe fn aligned_alloc(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let ptr = match layout.size() {
            0 => {
                return Ok(unsafe {
                    NonNull::new_unchecked(core::ptr::slice_from_raw_parts_mut(
                        invalid_mut(layout.align()),
                        0,
                    ))
                });
            }
            size => unsafe {
                ffi::rpmalloc_heap_aligned_alloc(self.inner.as_ptr(), layout.align(), size)
            },
        };

        if ptr.is_null() {
            handle_alloc_error(layout);
        }

        let slice = unsafe { slice::from_raw_parts(ptr.cast::<u8>(), layout.size()) };
        Ok(NonNull::from(slice))
    }

    pub unsafe fn aligned_calloc(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let ptr = match layout.size() {
            0 => {
                return Ok(unsafe {
                    NonNull::new_unchecked(core::ptr::slice_from_raw_parts_mut(
                        invalid_mut(layout.align()),
                        0,
                    ))
                });
            }
            size => unsafe {
                ffi::rpmalloc_heap_aligned_calloc(self.inner.as_ptr(), layout.align(), 1, size)
            },
        };

        if ptr.is_null() {
            handle_alloc_error(layout);
        }

        let slice = unsafe { slice::from_raw_parts(ptr.cast::<u8>(), layout.size()) };
        Ok(NonNull::from(slice))
    }

    pub unsafe fn deallocate(&mut self, ptr: NonNull<u8>, _: Layout) {
        unsafe { ffi::rpmalloc_heap_free(self.inner.as_ptr(), ptr.cast::<c_void>().as_mut()) };
    }

    pub unsafe fn aligned_realloc(
        &mut self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        // Simple grow from an empty ptr.
        if old_layout.size() == 0 {
            return unsafe { self.aligned_alloc(new_layout) };
        }

        let new_ptr = match new_layout.size() {
            0 => {
                return Ok(unsafe {
                    // Deallocate + zero-size alloc.
                    self.deallocate(ptr, old_layout);
                    NonNull::new_unchecked(core::ptr::slice_from_raw_parts_mut(
                        invalid_mut(new_layout.align()),
                        0,
                    ))
                });
            }
            size => unsafe {
                ffi::rpmalloc_heap_aligned_realloc(
                    self.inner.as_ptr(),
                    ptr.cast::<c_void>().as_mut(),
                    new_layout.align(),
                    size,
                    0,
                )
            },
        };

        if new_ptr.is_null() {
            handle_alloc_error(new_layout);
        }

        let slice = unsafe { slice::from_raw_parts(new_ptr.cast::<u8>(), new_layout.size()) };
        Ok(NonNull::from(slice))
    }
}

impl<A: Allocator> Drop for Heap<A> {
    fn drop(&mut self) {
        unsafe {
            ffi::rpmalloc_heap_free_all(self.inner.as_ptr());
            ffi::rpmalloc_heap_destroy(self.inner.as_ptr());
        }
    }
}

struct MemoryInterface<A: Allocator> {
    alloc: A,
    inner: ffi::rpmalloc_interface_t,
    _pin: PhantomPinned,
}

impl<A: Allocator> MemoryInterface<A> {
    fn new(alloc: A) -> MemoryInterface<A> {
        MemoryInterface {
            alloc,
            inner: ffi::rpmalloc_interface_t {
                context: null_mut(),
                memory_map: euk_alloc_memory_map::<A>,
                memory_commit: euk_alloc_memory_commit::<A>,
                memory_decommit: euk_alloc_memory_decommit::<A>,
                memory_unmap: euk_alloc_memory_unmap::<A>,
                map_fail_callback: euk_alloc_map_fail_callback::<A>,
                error_callback: euk_alloc_error_callback::<A>,
            },
            _pin: PhantomPinned,
        }
    }

    unsafe fn mmap(&mut self, size: size_t, alignment: size_t) -> *mut c_void {
        let align = max(alignment, MIN_ALIGN);
        let layout = unsafe { Layout::from_size_align_unchecked(size, align) };
        match self.alloc.allocate(layout) {
            Ok(ptr) => ptr.as_ptr().cast::<c_void>(),
            Err(_) => null_mut(),
        }
    }

    unsafe fn unmap(&mut self, address: *mut c_void, size: size_t) {
        let layout = unsafe { Layout::from_size_align_unchecked(size, MIN_ALIGN) };
        let ptr = unsafe { address.cast::<u8>().as_ref().unwrap() };
        unsafe { self.alloc.deallocate(NonNull::from(ptr), layout) };
    }
}

#[inline(always)]
fn invalid_mut<T>(addr: usize) -> *mut T {
    unsafe { std::mem::transmute(addr) }
}
