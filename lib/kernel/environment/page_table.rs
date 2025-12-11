use bitflags::bitflags;

bitflags! {
    pub struct Mode: u8 {
        const USER = 0b0001;
        const READ = 0b1000;
        const WRITE = 0b0001_0000;
        const EXECUTE = 0b0010_0000;
    }
}

pub trait PageTable {
    /// Returns a new page table that is mapped to the core kernel memory
    /// Does not need to be mapped to general user pages
    fn new_kernel_mapped() -> Self;

    /// SAFETY: This will move around pointers, so any living pointers needs to still be valid
    unsafe fn activate(pt: &Self);
    /// SAFETY: This will move around pointers, so any living pointers needs to still be valid
    unsafe fn deactivate();

    fn is_active(pt: &Self) -> bool;
    fn is_deactivated() -> bool;

    fn map(&mut self, virt: usize, phys: usize, mode: Mode);
}
