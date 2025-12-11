extern crate alloc;

use alloc::{collections::LinkedList, rc::Rc, sync::Arc};

use crate::{
    collections::mutex::Mutex,
    kernel::{
        environment::{Environment, Frame},
        mem::UserPages,
        process::{Process, ProcessId},
    },
};

mod task;
pub use task::*;

mod stack;
pub use stack::*;

#[derive(Debug)]
pub struct Scheduler<ENV: Environment> {
    tasks: LinkedList<Task<ENV>>,
}

impl<ENV: Environment> Scheduler<ENV> {
    pub fn new() -> Self {
        Self {
            tasks: LinkedList::new(),
        }
    }

    pub fn new_test_task(&mut self, id: usize, data: &[u8]) {
        let mut process = Process::from_slice(ProcessId::from(id), data);

        Rc::get_mut(&mut process.memory)
            .unwrap()
            .grow(UserPages::zeroed(40));

        let frame = ENV::Frame::empty(process.memory.user_start);

        self.add_task(Task {
            id: TaskId::new(id),
            frame: frame,
            pin: Pin::Unpinned,
            stack: Stack::new(),
            process: Arc::new(Mutex::new(process)),
        });
    }

    pub fn add_task(&mut self, task: Task<ENV>) {
        self.tasks.push_back(task);
    }

    pub fn next_task(&mut self, core: usize) -> Option<Task<ENV>> {
        let mut wrap_around: Option<TaskId> = None;
        loop {
            let task = self.tasks.pop_front();
            if let Some(task) = task {
                if let Some(wrap_around) = wrap_around {
                    if task.id == wrap_around {
                        self.tasks.push_back(task);
                        return None; // No more tasks to run
                    }
                } else {
                    wrap_around = Some(task.id);
                }

                if task.pin.can_take(core) {
                    return Some(task); 
                } else {
                    self.tasks.push_back(task);
                }
            } else {
                return None;
            }
        }
    }
}
