#![allow(non_snake_case, non_camel_case_types)]
#![allow(unsafe_code)] // FFI bindings need it
#![deny(missing_docs)]

pub use libc::{c_int, c_uint, c_void, size_t};

#[repr(C)]
#[cfg(feature = "first_class_heaps")]
pub struct rpmalloc_heap_t {
    _private: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct rpmalloc_interface_t {
    pub context: *mut c_void,
    /// Map memory pages for the given number of bytes. The returned address MUST be aligned to the given alignment,
    /// which will always be either 0 or the span size. The function can store an alignment offset in the offset
    /// variable in case it performs alignment and the returned pointer is offset from the actual start of the memory
    /// region due to this alignment. This alignment offset will be passed to the memory unmap function. The mapped size
    /// can be stored in the mapped_size variable, which will also be passed to the memory unmap function as the release
    /// parameter once the entire mapped region is ready to be released. If you set a memory_map function, you must also
    /// set a memory_unmap function or else the default implementation will be used for both. This function must be
    /// thread safe, it can be called by multiple threads simultaneously.
    pub memory_map: unsafe extern "C" fn(
        context: *mut c_void,
        size: size_t,
        alignment: size_t,
        offset: *mut size_t,
        mapped_size: *mut size_t,
    ) -> *mut c_void,
    /// Commit a range of memory pages
    pub memory_commit:
        unsafe extern "C" fn(context: *mut c_void, address: *mut c_void, size: size_t),
    /// Decommit a range of memory pages
    pub memory_decommit:
        unsafe extern "C" fn(context: *mut c_void, address: *mut c_void, size: size_t),
    /// Unmap the memory pages starting at address and spanning the given number of bytes. If you set a memory_unmap
    /// function, you must also set a memory_map function or else the default implementation will be used for both. This
    /// function must be thread safe, it can be called by multiple threads simultaneously.
    pub memory_unmap: unsafe extern "C" fn(
        context: *mut c_void,
        address: *mut c_void,
        offset: size_t,
        mapped_size: size_t,
    ),
    /// Called when a call to map memory pages fails (out of memory). If this callback is not set or returns zero the
    /// library will return a null pointer in the allocation call. If this callback returns non-zero the map call will
    /// be retried. The argument passed is the number of bytes that was requested in the map call. Only used if the
    /// default system memory map function is used (memory_map callback is not set).
    pub map_fail_callback: unsafe extern "C" fn(context: *mut c_void, size: size_t) -> c_int,
    /// Called when an assert fails, if asserts are enabled. Will use the standard assert() if this is not set.
    pub error_callback:
        unsafe extern "C" fn(context: *mut c_void, message: *const ::std::os::raw::c_char),
}

unsafe extern "C" {
    #[cfg(feature = "first_class_heaps")]
    /// Allocate memory for a new heap. Heap API is implemented with the strict assumption that only one single
    /// thread will call heap functions for a given heap at any given time, no functions are thread safe.
    pub fn rpmalloc_heap_create(
        memory_interface: *const rpmalloc_interface_t,
    ) -> *mut rpmalloc_heap_t;

    #[cfg(feature = "first_class_heaps")]
    /// Deallocate a heap (does NOT free the memory allocated by the heap, use rpmalloc_heap_free_all before destroying the
    /// heap).
    pub fn rpmalloc_heap_destroy(heap: *mut rpmalloc_heap_t);

    #[cfg(feature = "first_class_heaps")]
    /// Free all memory allocated by the heap
    pub fn rpmalloc_heap_free_all(heap: *mut rpmalloc_heap_t);

    #[cfg(feature = "first_class_heaps")]
    /// Allocate a memory block of at least the given size using the given heap. The returned
    /// block will have the requested alignment. Alignment must be a power of two and a multiple of sizeof(void*),
    /// and should ideally be less than memory page size. A caveat of rpmalloc
    /// internals is that this must also be strictly less than the span size (default 64KiB).
    pub fn rpmalloc_heap_aligned_alloc(
        heap: *mut rpmalloc_heap_t,
        alignment: size_t,
        size: size_t,
    ) -> *mut c_void;

    #[cfg(feature = "first_class_heaps")]
    /// Allocate a memory block of at least the given size using the given heap and zero initialize it. The returned
    /// block will have the requested alignment. Alignment must either be zero, or a power of two and a multiple of
    /// sizeof(void*), and should ideally be less than memory page size. A caveat of rpmalloc internals is that this must
    /// also be strictly less than the span size (default 64KiB).
    pub fn rpmalloc_heap_aligned_calloc(
        heap: *mut rpmalloc_heap_t,
        alignment: size_t,
        num: size_t,
        size: size_t,
    ) -> *mut c_void;

    #[cfg(feature = "first_class_heaps")]
    /// Reallocate the given block to at least the given size. The memory block MUST be allocated
    /// by the same heap given to this function. The returned block will have the requested alignment.
    /// Alignment must be either zero, or a power of two and a multiple of sizeof(void*), and should ideally be
    /// less than memory page size. A caveat of rpmalloc internals is that this must also be strictly less than
    /// the span size (default 64KiB).
    pub fn rpmalloc_heap_aligned_realloc(
        heap: *mut rpmalloc_heap_t,
        ptr: *mut c_void,
        alignment: size_t,
        size: size_t,
        flags: c_uint,
    ) -> *mut c_void;

    #[cfg(feature = "first_class_heaps")]
    /// Free the given memory block from the given heap. The memory block MUST be allocated
    /// by the same heap given to this function.
    pub fn rpmalloc_heap_free(heap: *mut rpmalloc_heap_t, ptr: *mut c_void);
}
