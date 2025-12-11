use crate::arch::riscv::sbi;

pub fn init() {
    schedule();
}

pub fn schedule() {
    let datel: u32;
    let dateh: u32;
    unsafe {
        core::arch::asm!(
            "
            rdtime {datel}
            rdtimeh {dateh}
            ",
            datel = out(reg) datel,
            dateh = out(reg) dateh,
            options(nostack, preserves_flags)
        );
    }
    let date = ((dateh as u64) << 32) | (datel as u64);
    sbi::timer::set_timer(date + 1000).unwrap();
}
