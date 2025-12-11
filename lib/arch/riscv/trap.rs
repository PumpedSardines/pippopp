use core::arch::asm;
use core::fmt::Write;

use crate::arch::riscv::csr::{Scause, Sepc, Sscratch, Sstatus};
use crate::arch::riscv::environment::riscv32im::EnvironmentRiscv32im;
use crate::arch::riscv::frame::riscv32im::FrameRiscv32im;
use crate::arch::riscv::{kernel, timer};
use crate::kernel::environment::Frame;
use crate::kernel::trap::{TrapCtx, TrapReason};
use crate::kernel::Kernel;

#[cfg(target_pointer_width = "32")]
pub unsafe fn trap_set_kernel(kernel: *mut Kernel<EnvironmentRiscv32im>) {
    Sscratch::new(kernel.addr() as u32).store();
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn supervisor_ecall(
    tf: *mut FrameRiscv32im,
    kernel: *const Kernel<EnvironmentRiscv32im>,
) -> ! {
    let frame = unsafe { &mut *tf };
    if frame.is_user_mode() {
        panic!("Called from user mode");
    }
    Sepc::new(frame.pc).store();
    let kernel = unsafe { &*kernel };
    kernel.trap(frame, TrapReason::KernelYield);
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn trap_entry(
    tf: *mut FrameRiscv32im,
    kernel: *const Kernel<EnvironmentRiscv32im>,
) -> ! {
    // SAFETY: This pointer comes from the stack from ASM, so the pointer is not refrenced
    // somewhere else
    let frame = unsafe { &mut *tf };
    let kernel = unsafe { &*kernel };

    let scause = Scause::load();
    let mut sstatus = Sstatus::load();
    let sepc = Sepc::load();

    if let Scause::Interrupt(code) = scause {
        match code {
            5 => {
                timer::schedule();
                kernel.trap(frame, TrapReason::Timer);
            }
            _ => {
                panic!("Unknown interrupt code: {}", code);
            }
        }
    }

    match scause {
        Scause::Ecall => {
            frame.pc += 4;
            Sepc::new(frame.pc).store();
            kernel.trap(
                frame,
                TrapReason::SysCall(
                    frame.a7 as usize,
                    frame.a0 as usize,
                    frame.a1 as usize,
                    frame.a2 as usize,
                ),
            );
        }
        Scause::SegFault => {
            let stval: u32;

            unsafe {
                asm!("csrr {}, stval", out(reg) stval);
            }

            kernel.trap(
                frame,
                TrapReason::SegFault {
                    pc_addr: frame.pc as usize,
                    addr: stval as usize,
                },
            );
        }
        Scause::Interrupt(_) => {
            unreachable!("Handle above");
        }
        Scause::Unknown(scause) => {
            let stval: u32;
            let sepc: u32;

            unsafe {
                asm!("csrr {}, stval", out(reg) stval);
                asm!("csrr {}, sepc", out(reg) sepc);
            }

            println!(
                "Unknown trap: scause={:x}, stval={:x}, sepc={:x}, core={}",
                scause, stval, sepc, kernel.core
            );
            panic!("Unknown trap");
        }
    }
}
