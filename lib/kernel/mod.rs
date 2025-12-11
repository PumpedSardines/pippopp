extern crate alloc;

use core::{
    arch::asm,
    cell::{Cell, RefCell, UnsafeCell},
    sync::atomic::{AtomicBool, Ordering},
};

use alloc::{
    alloc::{alloc, Layout},
    rc::Rc,
    sync::Arc,
};

pub mod environment;
pub mod mem;
pub mod process;
pub mod scheduler;
pub mod trap;

use crate::{
    arch::riscv::{csr::Sstatus, trap::trap_set_kernel},
    collections::mutex::Mutex,
    kernel::{
        environment::{Dispatch, DispatchLevel, Environment, Frame, PageTable},
        mem::UserPages,
        process::{Process, ProcessId, ProcessState},
        scheduler::{Pin, Scheduler, Stack, Task, TaskId},
    },
};

#[derive(Debug)]
#[repr(C)]
pub struct Kernel<ENV: Environment> {
    // These 2 need to be here due to asm
    pub scratch: UnsafeCell<[usize; 8]>,
    pub stack: UnsafeCell<*mut u8>,

    pub current_running: RefCell<Option<Task<ENV>>>,
    pub scheduler: Arc<Mutex<Scheduler<ENV>>>,
    pub core: usize,
}

impl<ENV: Environment> Kernel<ENV> {
    pub fn new(core: usize, scheduler: Arc<Mutex<Scheduler<ENV>>>) -> Self {
        Self {
            core,
            scratch: UnsafeCell::new([0; 8]),
            stack: UnsafeCell::new(core::ptr::null_mut()),
            current_running: RefCell::new(None),
            scheduler,
        }
    }

    pub fn init(&self) {}

    pub fn context_switch(&self, old_frame: &ENV::Frame) -> ! {
        ENV::Dispatch::deactivate_irq();
        let new_frame = {
            let (running_task, frame) = {
                let mut scheduler = self.scheduler.lock();
                let mut running_task = self.current_running.borrow_mut();
                match running_task.take() {
                    Some(mut task) => {
                        task.frame = old_frame.clone();
                        scheduler.add_task(task);
                    }
                    None => {}
                };

                let new_task = scheduler.next_task(self.core);
                let new_task = new_task.unwrap();
                let new_process = new_task.process.try_lock().unwrap();
                let frame = new_task.frame.clone();

                // Activate the page table of the running process
                unsafe {
                    ENV::PageTable::activate(&new_process.memory.page_table);
                    self.stack.get().write(new_task.stack.get_base() as *mut u8);
                }
                drop(new_process);
                (new_task, frame)
            };

            self.current_running.replace(Some(running_task));
            frame
        };
        unsafe {
            // TODO: Include this in the frame instead of hard coded here
            let mut sstatus = Sstatus::load();
            sstatus.SPIE = false;
            sstatus.store();
            ENV::Dispatch::dispatch(&new_frame);
        }
    }

    pub fn kernel_yield(&self) {
        ENV::Dispatch::deactivate_irq();
        {
            let mut running_task = self.current_running.borrow_mut();
            let running_task = running_task.as_mut().unwrap();
            running_task.pin = Pin::Core(self.core);
        }
        ENV::Dispatch::kernel_yield();
    }

    pub fn start(&self) -> ! {
        use crate::kernel::environment::Dispatch;
        use crate::kernel::environment::PageTable;
        ENV::Dispatch::deactivate_irq();

        println!("Kernel starting...");

        let frame = {
            let mut scheduler = self.scheduler.lock();
            let task = scheduler.next_task(self.core).unwrap();

            {
                let process = task.process.try_lock().unwrap();
                unsafe {
                    ENV::PageTable::activate(&process.memory.page_table);
                    self.stack.get().write(task.stack.get_base() as *mut u8);
                }
            }
            let frame = task.frame.clone();
            self.current_running.replace(Some(task));
            frame
        };

        unsafe {
            ENV::Dispatch::dispatch(&frame);
        }
    }
}
