#![cfg_attr(feature = "unstable-allocator-api", feature(allocator_api))]

mod heap;
mod ffi;

#[cfg(test)]
mod test;

pub use heap::Heap;
