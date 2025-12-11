//! Memory management module for the kernel.

pub mod page_allocator;
mod kernel_allocator;
mod pages;
pub use pages::*;

#[global_allocator]
pub static mut ALLOCATOR: kernel_allocator::KernelAllocator = kernel_allocator::KernelAllocator::new();

/// SAFETY: Caller ensures this function is only called once
pub unsafe fn init() {
    // SAFETY: Since this function can only be called once both of our init calls are safe
    unsafe {
        page_allocator::init();
        kernel_allocator::init();
    }
}
