use core::{
    arch::asm,
    sync::atomic::{AtomicPtr, Ordering::SeqCst},
};

pub(super) const PAGE_SIZE: usize = 4096;
const PAGE_COUNT: usize = 16 * 1024;

static HEAP_START: AtomicPtr<u8> = AtomicPtr::new(core::ptr::null_mut());
static HEAP_END: AtomicPtr<u8> = AtomicPtr::new(core::ptr::null_mut());

/// SAFETY: Caller must ensure that this function is called only once
pub(super) unsafe fn init() {
    let heap_start: *mut u8;
    unsafe {
        asm!(
            "la {}, __heap_begin",
            out(reg) heap_start
        );
    }
    assert_eq!((heap_start as usize) % PAGE_SIZE, 0);
    HEAP_START.store(heap_start, SeqCst);
    HEAP_END.store(
        heap_start.wrapping_add(PAGE_SIZE * PAGE_COUNT),
        SeqCst,
    );
}

pub fn alloc_pages(pages: usize) -> *mut u8 {
    loop {
        let heap_start = HEAP_START.load(SeqCst);
        if heap_start.is_null() {
            panic!("Heap not initialized");
        }
        let ptr = heap_start;
        let size = PAGE_SIZE * pages;
        let end = ptr.wrapping_add(size);
        if end >= HEAP_END.load(SeqCst) {
            panic!("Out of memory: page heap exhausted");
        }
        if HEAP_START
            .compare_exchange(heap_start, end, SeqCst, SeqCst)
            .is_ok()
        {
            unsafe { core::ptr::write_bytes(ptr, 0, size) };
            return ptr;
        }
    }
}

/// SAFETY: The caller must ensure that the they deallocate the correct number of pages
pub unsafe fn dealloc_pages(ptr: *mut u8, pages: usize) {
    // TODO: Implement a real allocator
}
