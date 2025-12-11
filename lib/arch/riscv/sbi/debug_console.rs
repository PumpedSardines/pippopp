use super::*;
use core::arch::{global_asm, naked_asm};
use core::fmt;

#[unsafe(naked)]
extern "C" fn sbi_debug_console_write(num_bytes: u32, addr_lo: u32, addr_hi: u32) -> SbiRet {
    naked_asm!(
        "
        li a7, 0x4442434E
        li a6, 0
        ecall
        ret
        "
    );
}

#[unsafe(naked)]
extern "C" fn sbi_debug_console_read(num_bytes: u32, addr_lo: u32, addr_hi: u32) -> SbiRet {
    naked_asm!(
        "
        li a7, 0x4442434E
        li a6, 1
        ecall
        ret
        "
    );
}

/// SAFETY: Callee needs to ensure debug console extension is enabled.
pub unsafe fn write(s: &[u8]) -> SbiResult {
    // I didn't get inline asm to work here
    // I think rust messes up the registers so everything gets clobbered
    let num_bytes = s.len() as u32;
    let base_addr = s.as_ptr() as u64;
    let base_addr_lo = base_addr as u32;
    let base_addr_hi = (base_addr >> 32) as u32;
    return sbi_debug_console_write(num_bytes, base_addr_lo, base_addr_hi).into_result();
}


unsafe fn read() -> SbiResult {
    // Need to rework this function, because i dont want it to work like this hmm lol
    let mut buf = [0u8; 1];
    let num_bytes = 1;
    let base_addr = buf.as_mut_ptr() as u64;
    let base_addr_lo = base_addr as u32;
    let base_addr_hi = (base_addr >> 32) as u32;
    sbi_debug_console_read(num_bytes, base_addr_lo, base_addr_hi).into_result()
}

pub struct SbiWriter;
impl fmt::Write for SbiWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        unsafe {
            write(s.as_bytes()).unwrap();
        }
        Ok(())
    }
}
