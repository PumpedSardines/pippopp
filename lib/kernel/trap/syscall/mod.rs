extern crate alloc;

use alloc::rc::Rc;

use crate::kernel::{
    environment::{Dispatch, DispatchLevel, Environment, Frame, PageTable},
    mem::UserPages,
    trap::TrapCtx,
    Kernel,
};

pub enum SystemCall {
    Yield,
    UartDebugPrint(char),
}

impl SystemCall {
    pub fn from_regs(r1: usize, r2: usize, r3: usize, r4: usize) -> Option<SystemCall> {
        match r1 {
            1 => Some(SystemCall::Yield),
            0 => {
                if let Some(c) = char::from_u32(r2 as u32) {
                    Some(SystemCall::UartDebugPrint(c))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

pub fn trap_syscall<ENV: Environment>(
    kernel: &Kernel<ENV>,
    kind: SystemCall,
    ctx: TrapCtx<'_, ENV>,
) -> ! {
    match kind {
        SystemCall::Yield => {
            unimplemented!();
        }
        SystemCall::UartDebugPrint(c) => {
            print!("{}", c);
            ENV::Dispatch::deactivate_irq();
            {
                let running_task = kernel.current_running.borrow_mut();
                let running_task = running_task.as_ref().unwrap();
                let running_process = running_task.process.try_lock().unwrap();
                unsafe {
                    ENV::PageTable::activate(&running_process.memory.page_table);
                    kernel
                        .stack
                        .get()
                        .write(running_task.stack.get_base().cast_mut());
                }
            }

            unsafe {
                ENV::Dispatch::dispatch(&ctx.frame);
            }
        }
    }
}
