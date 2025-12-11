extern crate alloc;

use crate::arch::riscv::csr::Satp;
use crate::kernel::environment::PageTable;
use crate::{kernel::environment::Mode, utils::is_aligned};
use alloc::alloc::{alloc, dealloc, Layout};

const SATP_SV32: u32 = 1 << 31;
const PAGE_V: u32 = 1 << 0;
const PAGE_R: u32 = 1 << 1;
const PAGE_W: u32 = 1 << 2;
const PAGE_X: u32 = 1 << 3;
const PAGE_U: u32 = 1 << 4;
const SATP_PAGE_SIZE: u32 = 4096;

#[derive(Debug)]
#[cfg(target_pointer_width = "32")]
struct SatpSv32Table0(*mut u32);

impl SatpSv32Table0 {
    pub fn new() -> Self {
        let layout = Layout::from_size_align(4096, 4096).unwrap();
        // SAFETY: Layout is non zero
        let ptr = unsafe { alloc(layout) as *mut u32 };
        if ptr.is_null() {
            panic!("Failed to allocate page table");
        }
        unsafe { core::ptr::write_bytes(ptr, 0, 1024) };
        Self(ptr)
    }

    pub fn map(&mut self, virt: u32, phys: u32, flags: u32) {
        let vpn0 = (virt as usize >> 12) & 0x3ff;
        let entry = unsafe { self.0.add(vpn0) };
        unsafe { entry.write(((phys >> 12) << 10) | flags) };
    }
}

#[derive(Debug)]
#[cfg(target_pointer_width = "32")]
pub struct SatpSv32Table1(*mut u32);

impl SatpSv32Table1 {
    pub const SATP_DEACTIVATED: u32 = 0;

    pub fn new() -> Self {
        let layout = Layout::from_size_align(4096, 4096).unwrap();
        // SAFETY: Layout is non zero
        let ptr = unsafe { alloc(layout) as *mut u32 };
        if ptr.is_null() {
            panic!("Failed to allocate page table");
        }
        unsafe { core::ptr::write_bytes(ptr, 0, 1024) };
        Self(ptr)
    }

    fn table0(&mut self, virt: u32) -> SatpSv32Table0 {
        let vpn1 = (virt as usize >> 22) & 0x3ff;
        let entry = unsafe { self.0.add(vpn1) };
        let raw_entry = unsafe { *entry };
        if raw_entry & PAGE_V == 0 {
            let table0 = unsafe { SatpSv32Table0::new() };
            let raw_entry = ((table0.0 as u32 >> 12) << 10) | PAGE_V;
            unsafe { core::ptr::write(entry, raw_entry) };
            table0
        } else {
            let table0 = ((raw_entry >> 10) << 12) as *mut u32;
            SatpSv32Table0(table0)
        }
    }

    pub fn as_satp(&self) -> u32 {
        let table1_addr = self.0 as u32 >> 12;
        SATP_SV32 | table1_addr
    }

    pub fn map(&mut self, virt: u32, phys: u32, flags: u32) {
        if !is_aligned(virt as usize, SATP_PAGE_SIZE as usize)
            || !is_aligned(phys as usize, SATP_PAGE_SIZE as usize)
        {
            panic!("virt and phys must be aligned to page size");
        }

        let mut table0 = self.table0(virt);
        table0.map(virt, phys, flags);
    }
}

#[cfg(target_pointer_width = "32")]
impl Drop for SatpSv32Table1 {
    fn drop(&mut self) {
        for i in 0..1024 {
            let entry = unsafe { self.0.add(i) };
            if unsafe { *entry } & PAGE_V != 0 {
                let table0 = ((unsafe { *entry } >> 10) << 12) as *mut u32;
                let layout = Layout::from_size_align(4096, 4096).unwrap();
                unsafe { dealloc(table0 as *mut u8, layout) };
            }
        }
        unsafe {
            let layout = Layout::from_size_align(4096, 4096).unwrap();
            dealloc(self.0 as *mut u8, layout);
        }
    }
}

#[cfg(target_pointer_width = "32")]
impl PageTable for SatpSv32Table1 {
    fn new_kernel_mapped() -> Self {
        let mut table1 = SatpSv32Table1::new();
        let mut kernel_begin: u32;
        let mut kernel_end: u32;

        unsafe {
            core::arch::asm!("
                la {kernel_base}, __kernel_begin
                la {kernel_end}, __kernel_end
            ",
                kernel_base = out(reg) kernel_begin,
                kernel_end = out(reg) kernel_end,
                options(nostack)
            );
        }


        let mut start = kernel_begin;
        while start < kernel_end {
            table1.map(start, start, PAGE_R | PAGE_W | PAGE_X | PAGE_V);
            start += SATP_PAGE_SIZE;
        }

        table1
    }

    unsafe fn activate(pt: &Self) {
        unsafe {
            core::arch::asm!("
                    sfence.vma
                    csrw satp, {satp}
                    sfence.vma
                ",
                satp = in(reg) pt.as_satp(),
                options(nostack)
            );
        }
    }

    unsafe fn deactivate() {
        unsafe {
            core::arch::asm!("
                sfence.vma
                csrw satp, {satp}
                sfence.vma
            ",
            satp = in(reg) Self::SATP_DEACTIVATED,
            options(nostack));
        }
    }

    fn is_active(pt: &Self) -> bool {
        let satp = Satp::load().as_usize() as u32;
        satp == pt.as_satp()
    }

    fn is_deactivated() -> bool {
        let satp = Satp::load().as_usize() as u32;
        return (satp & SATP_SV32) == 0
    }

    fn map(&mut self, virt: usize, phys: usize, mode: Mode) {
        let mut flags: u32 = PAGE_V;

        if mode.contains(Mode::USER) {
            flags |= PAGE_U;
        }

        if mode.contains(Mode::READ) {
            flags |= PAGE_R;
        }

        if mode.contains(Mode::WRITE) {
            flags |= PAGE_W;
        }

        if mode.contains(Mode::EXECUTE) {
            flags |= PAGE_X;
        }

        SatpSv32Table1::map(self, virt as u32, phys as u32, flags);
    }
}
