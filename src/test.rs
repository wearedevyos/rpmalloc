use crate::Heap;
use allocator_api2::alloc::Global;
use std::alloc::Layout;
use std::ptr::NonNull;

#[test]
fn test_alloc_dealloc_in_global() {
    let mut heap = Heap::<Global>::try_new().expect("failed to create heap structure");

    let layout = Layout::new::<u32>();

    let data_ptr = unsafe { heap.aligned_alloc(layout) }.expect("u32 allocation failure");
    let raw_ptr = data_ptr.as_ptr().cast::<u8>();

    unsafe { heap.deallocate(NonNull::new_unchecked(raw_ptr), layout) };
}
