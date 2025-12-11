#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            let mut writer = crate::arch::riscv::sbi::debug_console::SbiWriter;
            write!(writer, $($arg)*).unwrap();
        }
    };
}

#[macro_export]
macro_rules! println {
    () => {
        print!("\n")
    };
    ($($arg:tt)*) => {
        print!($($arg)*);
        print!("\n");
    };
}
