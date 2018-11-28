use super::task::Task;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub struct Queue<T> {
    inner: Arc<Mutex<VecDeque<T>>>,
}

impl<T> Queue<T> {
    pub fn new() -> Queue<T> {
        Queue {
            inner: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn push(&mut self, item: T) {
        self.inner.lock().unwrap().push_back(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        let mut mutex = self.inner.lock().unwrap();

        if mutex.len() == 0 {
            return None;
        }

        mutex.pop_front()
    }

    pub fn len(&self) -> usize {
        self.inner.lock().unwrap().len()
    }
}

pub type TaskQueue = Arc<RefCell<Queue<Task>>>;

pub fn createTaskQueue() -> TaskQueue {
    Arc::new(RefCell::new(Queue::new()))
}
