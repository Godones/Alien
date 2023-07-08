use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::cmp::min;

use crate::error::{AlienError, AlienResult};
use crate::task::{Task, TASK_MANAGER};
use crate::timer::read_timer;

pub struct FutexWaiter {
    task: Option<Arc<Task>>,
    is_wait: bool,
    wait_time: usize,
}

pub struct FutexWaitManager {
    map: BTreeMap<usize, Vec<FutexWaiter>>,
}

impl FutexWaiter {
    pub fn new(task: Arc<Task>, wait_time: usize) -> Self {
        Self {
            task: Some(task),
            is_wait: false,
            wait_time,
        }
    }

    pub fn is_wait(&self) -> bool {
        self.is_wait || read_timer() < self.wait_time
    }

    pub fn wake(&mut self) -> Arc<Task> {
        self.is_wait = false;
        self.task.take().unwrap()
    }
}

impl FutexWaitManager {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }
    pub fn add_waiter(&mut self, futex: usize, waiter: FutexWaiter) {
        self.map.entry(futex).or_insert(Vec::new()).push(waiter);
    }

    pub fn wake(&mut self, futex: usize, num: usize) -> AlienResult<usize> {
        if let Some(waiters) = self.map.get_mut(&futex) {
            let min_index = min(num, waiters.len());
            for i in 0..min_index {
                if !waiters[i].is_wait() {
                    let task = waiters[i].wake();
                    let mut task_manager = TASK_MANAGER.lock();
                    task_manager.push_back(task);
                }
            }
            Ok(min_index)
        } else {
            Err(AlienError::Other)
        }
    }
}
