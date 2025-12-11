extern crate alloc;

use alloc::alloc::{alloc, dealloc, Layout};
use alloc::boxed::Box;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GrowDirection {
    Down,
    Up,
}

#[derive(Debug)]
pub struct Stack(*const u8);

impl Stack {
    const SIZE: usize = 8 * 1024;

    pub fn new() -> Self {
        let data = unsafe {
            let layout = Layout::from_size_align(Self::SIZE, Self::alignment()).unwrap();
            alloc(layout)
        };
        if data.is_null() {
            panic!("Failed to allocate stack memory");
        }
        Stack(data)
    }

    #[cfg(target_arch = "riscv32")]
    fn alignment() -> usize {
        4
    }

    #[cfg(target_arch = "riscv32")]
    fn size_from_len(len: usize) -> usize {
        len * 4
    }

    #[cfg(target_arch = "riscv32")]
    const fn grow_direction() -> GrowDirection {
        GrowDirection::Down
    }

    pub fn get_base(&self) -> *const u8 {
        match Self::grow_direction() {
            GrowDirection::Down => unsafe { self.0.add(Self::SIZE) },
            GrowDirection::Up => self.0,
        }
    }
}
