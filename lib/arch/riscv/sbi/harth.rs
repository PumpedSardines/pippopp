use core::arch::naked_asm;

use crate::arch::riscv::sbi::{SbiResult, SbiRet};

#[unsafe(naked)]
extern "C" fn sbi_harth_start(hartid: u32, start_addr: u32, opaque: u32) -> SbiRet {
    naked_asm!(
        "
        li a7, 0x48534D
        li a6, 0
        ecall
        ret
        "
    );
}

#[unsafe(naked)]
extern "C" fn sbi_harth_stop(hartid: u32) -> SbiRet {
    naked_asm!(
        "
        li a7, 0x48534D
        li a6, 0
        ecall
        ret
        "
    );
}

#[unsafe(naked)]
extern "C" fn sbi_harth_status(hartid: u32) -> SbiRet {
    naked_asm!(
        "
        li a7, 0x48534D
        li a6, 2
        ecall
        ret
        "
    );
}

pub fn start(hartid: u32, start_addr: *const (), opaque: u32) -> SbiResult {
    unsafe { sbi_harth_start(hartid, start_addr as u32, opaque).into_result() }
}

pub fn status(hartid: u32) -> SbiResult {
    unsafe { sbi_harth_status(hartid).into_result() }
}
