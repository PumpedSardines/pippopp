extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

pub struct RingBuffer<T> {
    buffer: Vec<T>,
    head: usize,
    tail: usize,
    capacity: usize,
}

impl<T: Clone + Default> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        RingBuffer {
            buffer: vec![T::default(); capacity],
            head: 0,
            tail: 0,
            capacity,
        }
    }

    pub fn push(&mut self, item: T) -> Result<(), ()> {
        if (self.tail + 1) % self.capacity == self.head {
            return Err(());
        }
        self.buffer[self.tail] = item;
        self.tail = (self.tail + 1) % self.capacity;
        Ok(())
    }

    pub fn pop(&mut self) -> Result<T, ()> {
        if self.head == self.tail {
            return Err(());
        }
        let item: T;
        item = self.buffer[self.head].clone();
        self.head = (self.head + 1) % self.capacity;
        Ok(item)
    }

    pub fn is_empty(&self) -> bool {
        self.head == self.tail
    }

    pub fn is_full(&self) -> bool {
        (self.tail + 1) % self.capacity == self.head
    }
}
