use core::cell::UnsafeCell;

use super::page_allocator::{alloc_pages, dealloc_pages, PAGE_SIZE};

/// Represents a block of pages allocated in user memeory space
#[derive(Debug)]
pub struct UserPages {
    ptr: *const u8,
    count: usize,
}

impl UserPages {
    pub const PAGE_SIZE: usize = PAGE_SIZE;

    /// Creates a new uninitialized block of pages
    pub fn new(count: usize) -> Self {
        let ptr = alloc_pages(count);
        if ptr.is_null() {
            panic!("Failed to allocate pages");
        }
        Self { ptr, count }
    }

    /// Creates a new block of pages with a minimum count as to fit a certain size
    pub fn with_capacity(size: usize) -> Self {
        let count = (size + PAGE_SIZE - 1) / PAGE_SIZE; // Round up to nearest page
        let ptr = unsafe { alloc_pages(count) };
        if ptr.is_null() {
            panic!("Failed to allocate pages");
        }
        Self { ptr, count }
    }

    /// Clears the contents of the pages by writing zeros to them
    pub fn clear(&mut self) {
        if !self.ptr.is_null() {
            /// SAFETY: Since we borrow self as &mut we have exclusive access to this memeory
            /// region
            unsafe {
                core::ptr::write_bytes(self.ptr_as_mut(), 0, self.count * PAGE_SIZE)
            };
        }
    }

    /// Creates a new block of pages initialized to zero
    pub fn zeroed(count: usize) -> Self {
        let ptr = unsafe { alloc_pages(count) };
        if ptr.is_null() {
            panic!("Failed to allocate pages");
        }
        unsafe {
            core::ptr::write_bytes(ptr, 0, count * PAGE_SIZE);
        }
        Self { ptr, count }
    }

    fn ptr_as_mut(&mut self) -> *mut u8 {
        // SAFETY: Since we borrow self as &mut we have exclusive access to this memory region
        unsafe { self.ptr as *mut u8 }
    }

    /// Creates a new `Pages` instance from raw parts where ptr is the start of the memory block
    /// and count is the amount of pages
    ///
    /// # Safety
    /// - The caller must ensure that `ptr` is aligned to `PAGE_SIZE`
    /// - The caller must ensure that the memory region is allocated and valid
    /// - The caller must ensure that the memory region is in user space
    /// - The caller must ensure that the memory region is not being refrenced anywhere else
    pub unsafe fn from_raw_parts(ptr: *mut u8, count: usize) -> Self {
        Self { ptr, count }
    }

    /// Consumes the `Pages` instance and returns the raw pointer and count
    /// This prevents the `Pages` instance from being dropped and deallocated
    pub fn into_raw(mut self) -> (*mut u8, usize) {
        let ptr = self.ptr_as_mut();
        // Prevent deallocation in drop
        self.ptr = core::ptr::null_mut();
        (ptr, self.count)
    }

    pub fn empty() -> Self {
        Self {
            ptr: core::ptr::null_mut(),
            count: 0,
        }
    }

    /// Returns the amount of pages allocated for this instance
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns the size in bytes of the allocated pages
    pub fn size(&self) -> usize {
        self.count * PAGE_SIZE
    }

    /// Returns a pointer to the start of the allocated pages
    pub fn start(&self) -> *const u8 {
        self.ptr
    }

    /// Returns a pointer to a specific page by index
    pub fn page(&self, index: usize) -> Option<*const u8> {
        if index >= self.count {
            return None;
        }
        Some(unsafe { self.ptr.add(index * PAGE_SIZE) })
    }

    pub fn start_mut(&mut self) -> *mut u8 {
        self.ptr_as_mut()
    }

    pub fn page_mut(&mut self, index: usize) -> Option<*mut u8> {
        if index >= self.count {
            return None;
        }
        Some(unsafe { self.ptr_as_mut().add(index * PAGE_SIZE) })
    }

    pub fn iter(&self) -> UserPagesRefIter {
        UserPagesRefIter {
            pages: self,
            index: 0,
        }
    }
}

impl Drop for UserPages {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { dealloc_pages(self.ptr_as_mut(), self.count) };
        }
    }
}

pub struct UserPagesRefIter<'a> {
    pages: &'a UserPages,
    index: usize,
}

impl<'a> Iterator for UserPagesRefIter<'a> {
    type Item = *const u8;

    fn next(&mut self) -> Option<Self::Item> {
        let page = self.pages.page(self.index);
        self.index += 1;
        page
    }
}
