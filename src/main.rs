#![feature(generic_const_exprs)]
#![no_std]
#![no_main]

mod arch;

use core::fmt::Write;
use core::panic::PanicInfo;

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use pippopp::arch::riscv as r32;
    let mut writer = r32::sbi::debug_console::SbiWriter;
    writeln!(writer, "{}", info).unwrap();
    loop {}
}
