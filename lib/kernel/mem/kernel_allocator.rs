use core::alloc::{GlobalAlloc, Layout};
use core::arch::asm;
use core::sync::atomic::AtomicPtr;
use core::sync::atomic::Ordering::SeqCst;

use crate::utils::align_up;

static HEAP_START: AtomicPtr<u8> = AtomicPtr::new(core::ptr::null_mut());
static HEAP_END: AtomicPtr<u8> = AtomicPtr::new(core::ptr::null_mut());

/// SAFETY: Caller must ensure that this function is called only once
pub(super) unsafe fn init() {
    let heap_start: *mut u8;
    let heap_end: *mut u8;
    unsafe {
        asm!(
            "la {}, __kernel_heap_begin",
            out(reg) heap_start
        );
        asm!(
            "la {}, __kernel_heap_end",
            out(reg) heap_end
        );
    }
    HEAP_START.store(heap_start, SeqCst);
    HEAP_END.store(heap_end, SeqCst);
}

pub struct KernelAllocator {}

impl KernelAllocator {
    pub const fn new() -> Self {
        Self {}
    }
}

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        loop {
            let heap_start = HEAP_START.load(SeqCst);
            let ptr = align_up(heap_start as usize, layout.align()) as *mut u8;
            let size = layout.size();
            let end = ptr.wrapping_add(size);

            if end > HEAP_END.load(SeqCst) {
                panic!("out of memeory")
            }

            if HEAP_START
                .compare_exchange(heap_start, end, SeqCst, SeqCst)
                .is_ok()
            {
                if ptr.is_null() {
                    panic!("Couldnt allocate");
                }
                unsafe { core::ptr::write_bytes(ptr, 0, size) };
                return ptr;
            }
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // TODO: Implement a real allocator
    }
}
