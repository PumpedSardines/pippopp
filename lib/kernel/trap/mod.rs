use core::sync::atomic::{AtomicUsize, Ordering::SeqCst};

use crate::{
    arch::riscv::csr::Sstatus,
    kernel::{
        environment::{Dispatch, DispatchLevel, Environment, Frame, PageTable},
        process::ProcessId,
        scheduler::Pin,
        Kernel,
    },
};

pub mod syscall;

#[derive(Debug)]
pub enum TrapReason {
    SysCall(usize, usize, usize, usize),
    SegFault { pc_addr: usize, addr: usize },
    Timer,
    KernelYield,
}

pub struct TrapCtx<'a, ENV: Environment> {
    pub frame: &'a mut ENV::Frame,
    pub reason: TrapReason,
}

static TIMER_INTERRUPTS: AtomicUsize = AtomicUsize::new(0);

impl<ENV: Environment> Kernel<ENV> {
    pub fn trap(&self, frame: &mut ENV::Frame, reason: TrapReason) -> ! {
        let ctx = TrapCtx { frame, reason };

        match ctx.reason {
            TrapReason::SysCall(r1, r2, r3, r4) => {
                ENV::Dispatch::activate_irq();
                let v = syscall::SystemCall::from_regs(r1, r2, r3, r4).unwrap();
                syscall::trap_syscall(self, v, ctx)
            }
            TrapReason::SegFault { pc_addr, addr } => {
                panic!(
                    "Seg fault: frame {:#x?} 0x{:x} 0x{:x}, core: {}",
                    frame, pc_addr, addr, self.core
                );
            }
            TrapReason::KernelYield => {
                self.context_switch(ctx.frame);
            }
            TrapReason::Timer => {
                let old = TIMER_INTERRUPTS.fetch_add(1, SeqCst);
                if old > 1000 {
                    TIMER_INTERRUPTS.store(0, SeqCst);
                    if ctx.frame.is_user_mode() {
                        ENV::Dispatch::deactivate_irq();
                        {
                            let mut running_task = self.current_running.borrow_mut();
                            let running_task = running_task.as_mut().unwrap();
                            running_task.pin = Pin::Unpinned;
                        }
                        self.context_switch(ctx.frame);
                    } else {
                        ENV::Dispatch::deactivate_irq();
                        self.kernel_yield();
                        {
                            let running_task = self.current_running.borrow_mut();
                            let running_task = running_task.as_ref().unwrap();
                            let running_process = running_task.process.try_lock().unwrap();
                            unsafe {
                                ENV::PageTable::activate(&running_process.memory.page_table);
                                self.stack
                                    .get()
                                    .write(running_task.stack.get_base().cast_mut());
                            }
                        };
                        unsafe {
                            ENV::Dispatch::dispatch(&ctx.frame);
                        }
                    }
                } else {
                    unsafe {
                        ENV::Dispatch::dispatch(&ctx.frame);
                    }
                }
            }
        }
    }
}
