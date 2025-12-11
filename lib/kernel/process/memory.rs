extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use crate::kernel::environment::Environment;
use crate::kernel::environment::Mode;
use crate::kernel::environment::PageTable;
use crate::kernel::mem::UserPages;

#[derive(Debug)]
pub struct Memory<ENV: Environment> {
    pub page_table: ENV::PageTable,
    pub pages: Vec<UserPages>,
    pub user_start: usize,
    pub user_end: usize,
}

impl<ENV: Environment> Memory<ENV> {
    const USER_START: usize = 0xC000_0000;

    pub fn new() -> Self {
        Memory {
            page_table: ENV::PageTable::new_kernel_mapped(),
            pages: vec![],
            user_start: Self::USER_START,
            user_end: Self::USER_START,
        }
    }

    /// Safety: The caller must ensure that no other references to the slice exist
    pub unsafe fn slice(&self, ptr: *mut u8, len: usize) -> Result<&[u8], ()> {
        if (ptr as usize) < self.user_start || (ptr as usize) + len > self.user_end {
            return Err(());
        }

        return unsafe {
            let slice = core::slice::from_raw_parts(ptr, len);
            Ok(slice)
        };
    }

    /// Safety: The caller must ensure that no other references to the slice exist
    pub unsafe fn slice_mut(&mut self, ptr: *mut u8, len: usize) -> Result<&mut [u8], ()> {
        if (ptr as usize) < self.user_start || (ptr as usize) + len > self.user_end {
            return Err(());
        }

        return unsafe {
            let slice = core::slice::from_raw_parts_mut(ptr, len);
            Ok(slice)
        };
    }

    pub fn grow(&mut self, pages: UserPages) {
        for (i, page) in pages.iter().enumerate() {
            self.page_table.map(
                self.user_end,
                page as usize,
                Mode::WRITE | Mode::READ | Mode::USER | Mode::EXECUTE,
            );
            self.user_end += UserPages::PAGE_SIZE;
        }
        self.pages.push(pages);
    }
}

impl<ENV: Environment> Clone for Memory<ENV> {
    fn clone(&self) -> Self {
        let count = self.pages.iter().fold(0, |acc, p| acc + p.len());
        let mut pages = UserPages::new(count);

        let mut j = 0;
        for page in self.pages.iter() {
            unsafe {
                core::ptr::copy_nonoverlapping(
                    page.page(0).unwrap(),
                    pages.page_mut(j).unwrap(),
                    page.size(),
                );
            }
            j += page.len();
        }

        let mut table = ENV::PageTable::new_kernel_mapped();

        for (i, page) in pages.iter().enumerate() {
            table.map(
                Self::USER_START + UserPages::PAGE_SIZE * i,
                page as usize,
                Mode::WRITE | Mode::READ | Mode::USER | Mode::EXECUTE,
            );
        }

        Memory {
            pages: vec![pages],
            page_table: table,
            user_start: self.user_start,
            user_end: self.user_end,
        }
    }
}

impl<ENV: Environment> Default for Memory<ENV> {
    fn default() -> Self {
        Self::new()
    }
}

impl<ENV: Environment> From<&[u8]> for Memory<ENV> {
    fn from(slice: &[u8]) -> Self {
        let mut pages = UserPages::with_capacity(slice.len());

        unsafe {
            core::ptr::copy_nonoverlapping(slice.as_ptr(), pages.start_mut(), slice.len());
        };

        let mut table = ENV::PageTable::new_kernel_mapped();
        for (i, page) in pages.iter().enumerate() {
            table.map(
                Self::USER_START + UserPages::PAGE_SIZE * i,
                page as usize,
                Mode::WRITE | Mode::READ | Mode::USER | Mode::EXECUTE,
            );
        }

        let user_end = Self::USER_START + pages.size();

        Memory {
            page_table: table,
            pages: vec![pages],
            user_start: Self::USER_START,
            user_end: user_end,
        }
    }
}
