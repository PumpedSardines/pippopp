use super::*;
use core::arch::{global_asm, naked_asm};

#[unsafe(naked)]
extern "C" fn sbi_base_get_spec_version() -> SbiRet {
    naked_asm!(
        "
        li a7, 0x10
        li a6, 0
        ecall
        ret
        "
    );
}

#[unsafe(naked)]
extern "C" fn sbi_base_get_impl_id() -> SbiRet {
    naked_asm!(
        "
        li a7, 0x10
        li a6, 1
        ecall
        ret
        "
    );
}

#[unsafe(naked)]
extern "C" fn sbi_base_probe_extension(extension_id: u32) -> SbiRet {
    naked_asm!(
        "
        li a7, 0x10
        li a6, 3
        ecall
        ret
        "
    );
}

// SAFETY: All of the following functions are guaranteed to exist in the SBI spec.

pub fn get_spec_version() -> SbiResult {
    return unsafe { sbi_base_get_spec_version().into_result() };
}

pub fn get_impl_id() -> SbiResult {
    return unsafe { sbi_base_get_impl_id().into_result() };
}

pub fn probe_extension(extension_id: u32) -> SbiResult {
    return unsafe { sbi_base_probe_extension(extension_id).into_result() };
}
