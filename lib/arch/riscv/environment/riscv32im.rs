use core::arch::naked_asm;

use crate::{
    arch::riscv::{
        dispatch::riscv32im::DispatcherRiscv32im, frame::riscv32im::FrameRiscv32im,
        page_table::satp_sv32::SatpSv32Table1,
    },
    kernel::environment::{Dispatch, Environment},
};

#[derive(Debug, Clone)]
pub struct EnvironmentRiscv32im {}

impl Environment for EnvironmentRiscv32im {
    type PageTable = SatpSv32Table1;
    type Frame = FrameRiscv32im;
    type Dispatch = DispatcherRiscv32im;

    fn wfi_program() -> &'static [u8] {
        const WFI: u32 = 0x0000006f;
        const WFI_PROGRAM: [u8; 4] = WFI.to_le_bytes();
        &WFI_PROGRAM
    }
}
