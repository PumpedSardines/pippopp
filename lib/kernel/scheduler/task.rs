extern crate alloc;

use alloc::sync::Arc;

use crate::{
    collections::mutex::Mutex,
    kernel::{
        environment::Environment,
        process::{Process, ProcessId},
        scheduler::Stack,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskId(usize);

impl From<usize> for TaskId {
    fn from(id: usize) -> Self {
        TaskId(id)
    }
}

impl TaskId {
    pub fn new(id: usize) -> Self {
        TaskId(id)
    }

    pub fn id(&self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Pin {
    Core(usize),
    Unpinned,
}

impl Pin {
    pub fn can_take(&self, core: usize) -> bool {
        match self {
            Pin::Core(c) => *c == core,
            Pin::Unpinned => true,
        }
    }
}

#[derive(Debug)]
pub struct Task<ENV: Environment> {
    pub id: TaskId,
    pub pin: Pin,
    pub frame: ENV::Frame,
    pub stack: Stack,
    pub process: Arc<Mutex<Process<ENV>>>,
}

impl<ENV: Environment> Task<ENV> {
    pub fn pin(&mut self, pin: Pin) {
        self.pin = pin;
    }
}
