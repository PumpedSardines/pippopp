extern crate alloc;

use core::sync::atomic::{AtomicBool, Ordering};

use alloc::rc::Rc;

use crate::{
    collections::mutex::Mutex,
    kernel::{
        environment::{Environment, Frame},
        process::memory::Memory,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessId(usize);

impl ProcessId {
    pub fn from(id: usize) -> Self {
        ProcessId(id)
    }

    pub fn as_usize(&self) -> usize {
        self.0
    }
}

impl core::fmt::Display for ProcessId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<usize> for ProcessId {
    fn from(id: usize) -> Self {
        ProcessId(id)
    }
}

#[derive(Debug)]
pub struct Process<ENV: Environment> {
    pub id: ProcessId,
    pub state: Mutex<ProcessState>,
    pub memory: Rc<Memory<ENV>>,
}

#[derive(Debug, Clone)]
pub enum ProcessState {
    Running,
    Idle,
    Exited,
}

impl<ENV: Environment> Process<ENV> {
    pub fn from_slice(id: ProcessId, slice: &[u8]) -> Self {
        let memory = Rc::new(Memory::from(slice));

        Process {
            id: id,
            state: Mutex::new(ProcessState::Idle),
            memory: memory,
        }
    }

    pub fn fork(&self, id: ProcessId) -> Process<ENV> {
        Process {
            id: id,
            state: Mutex::new(ProcessState::Idle),
            memory: Rc::new(self.memory.as_ref().clone()),
        }
    }

    pub fn new_wfi() -> Self {
        Process::from_slice(ProcessId::from(0), ENV::wfi_program())
    }
}

impl<ENV: Environment> Drop for Process<ENV> {
    fn drop(&mut self) {
        println!("[kernel] Dropping process");
    }
}
