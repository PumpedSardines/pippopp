extern crate alloc;

use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::sync::Arc;
use alloc::vec::Vec;

use pippopp::arch::riscv::environment::riscv32im::EnvironmentRiscv32im;
use pippopp::arch::riscv::sbi;
use pippopp::arch::riscv::trap::trap_set_kernel;
use pippopp::collections::mutex::Mutex;
use pippopp::kernel::scheduler::Scheduler;
use pippopp::kernel::Kernel;

use core::alloc::Layout;
use core::arch::naked_asm;
use core::fmt::Write;

use core::{
    arch::global_asm,
    sync::atomic::{fence, Ordering},
};
use pippopp::{
    arch::riscv::{
        csr::{Sstatus, Stvec},
        frame::riscv32im::FrameRiscv32im,
        kernel,
    },
    kernel::{
        environment::Frame,
        mem::UserPages,
        process::{Process, ProcessId},
        scheduler::{Stack, Task},
    },
};

global_asm!(include_str!("boot.S"));

const A_PROGRAM: &[u8] = include_bytes!("./A_main.bin");
// const PRIME_PROGRAM: &[u8] = include_bytes!("../../../exe/simple/main.bin");
const B_PROGRAM: &[u8] = include_bytes!("./B_main.bin");

static mut SCHEDULER: Option<Arc<Mutex<Scheduler<EnvironmentRiscv32im>>>> = None;
static LOCK: Mutex<()> = Mutex::new(());

#[no_mangle]
pub extern "C" fn kernel_main(a0: u32, a1: u32, a2: u32) -> ! {
    let entry_harth_id = a0;
    unsafe { pippopp::kernel::mem::init() };

    fence(Ordering::SeqCst);

    let mut scheduler = Scheduler::<EnvironmentRiscv32im>::new();
    scheduler.new_test_task(0, A_PROGRAM);
    scheduler.new_test_task(1, A_PROGRAM);
    scheduler.new_test_task(2, B_PROGRAM);
    scheduler.new_test_task(3, B_PROGRAM);
    scheduler.new_test_task(4, A_PROGRAM);
    scheduler.new_test_task(5, A_PROGRAM);
    scheduler.new_test_task(6, B_PROGRAM);
    scheduler.new_test_task(7, B_PROGRAM);
    // scheduler.new_test_task(2, PRIME_PROGRAM);
    // scheduler.new_test_task(2, PRIME_PROGRAM);

    unsafe {
        SCHEDULER = Some(Arc::new(Mutex::new(scheduler)));
    }

    let mut amount_started = 0;

    for i in 0..4 {
        use pippopp::arch::riscv as r32;
        let mut writer = r32::sbi::debug_console::SbiWriter;
        if i == entry_harth_id {
            writeln!(writer, "Main harth, skipping").unwrap();
            continue;
        } else {
            writeln!(writer, "Starting {}", i).unwrap();
        }

        let trampoline_stack = unsafe {
            let layout = Layout::from_size_align(0x1000, 0x1000).unwrap();
            let stack = alloc::alloc::alloc(layout) as *mut u8;
            stack.addr() + 0x1000 - 4
        };
        writeln!(writer, "Trampoline stack {:x}", trampoline_stack).unwrap();
        amount_started += 1;
        sbi::harth::start(
            i,
            harth_entrypoint_trampoline as *const (),
            trampoline_stack as u32,
        )
        .unwrap();
        if amount_started == 4 {
            writeln!(writer, "Started all harths").unwrap();
            break;
        }
    }
    loop {}
}

#[no_mangle]
#[unsafe(naked)]
pub extern "C" fn harth_entrypoint_trampoline(other: u32, opaque: u32) -> ! {
    naked_asm!(
        "
        mv s0, a0
        mv sp, a1 
        la gp, __global_pointer
        call _set_up_irq
        mv a0, s0
        call harth_entrypoint
        1: j 1b
    "
    )
}

#[no_mangle]
pub extern "C" fn harth_entrypoint(core: u32, opaque: u32) -> ! {
    use pippopp::arch::riscv as r32;
    let mut writer = r32::sbi::debug_console::SbiWriter;
    writeln!(writer, "Here").unwrap();

    let mut sstatus = Sstatus::load();
    sstatus.SUM = true;
    sstatus.SIE = false;
    sstatus.store();

    pippopp::arch::riscv::timer::init();

    let kernel = unsafe {
        alloc::alloc::alloc(Layout::new::<Kernel<EnvironmentRiscv32im>>())
            as *mut Kernel<EnvironmentRiscv32im>
    };
    let scheduler = unsafe { SCHEDULER.as_ref().unwrap().clone() };
    unsafe { kernel.write(Kernel::<EnvironmentRiscv32im>::new(core as usize, scheduler)) };
    unsafe { trap_set_kernel(kernel) };
    let kernel = unsafe { &*kernel };
    kernel.start();
}
