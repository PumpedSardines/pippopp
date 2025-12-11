use super::*;
use core::arch::naked_asm;

#[unsafe(naked)]
extern "C" fn sbi_timer_set_timer(stime_valueh: u32, stime_valuel: u32) -> SbiRet {
    naked_asm!(
        "
        li a7, 0x54494D45
        li a6, 0
        ecall
        ret
        "
    );
}

pub fn set_timer(stime_value: u64) -> SbiResult {
    let stime_valueh = (stime_value >> 32) as u32;
    let stime_valuel = (stime_value & 0xFFFFFFFF) as u32;
    return unsafe { sbi_timer_set_timer(stime_valuel, stime_valueh).into_result() };
}
