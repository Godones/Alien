use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::min;

use kernel_sync::Mutex;

use crate::error::{AlienError, AlienResult};
use crate::task::{Task, TASK_MANAGER};
use crate::timer::read_timer;

pub struct FutexWaiter {
    task: Option<Arc<Task>>,
    wait_time: Option<usize>,
    timeout_flag: Arc<Mutex<bool>>,
}

pub struct FutexWaitManager {
    map: BTreeMap<usize, Vec<FutexWaiter>>,
}

impl FutexWaiter {
    pub fn new(task: Arc<Task>, wait_time: Option<usize>, timeout_flag: Arc<Mutex<bool>>) -> Self {
        Self {
            task: Some(task),
            wait_time,
            timeout_flag,
        }
    }

    pub fn wake(&mut self) -> Arc<Task> {
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
    pub fn wake_for_signal(&mut self) {
        for (_, waiters) in self.map.iter_mut() {
            let mut record = vec![];
            for (index, waiter) in waiters.iter_mut().enumerate() {
                let task = waiter.task.as_ref().unwrap();
                let task_inner = task.access_inner();
                let receiver = task_inner.signal_receivers.lock();
                if receiver.have_signal() {
                    drop(receiver);
                    drop(task_inner);
                    let mut task_manager = TASK_MANAGER.lock();
                    let task = waiter.wake();
                    task_manager.push_back(task);
                    record.push(index);
                }
            }
            record.iter().for_each(|index| {
                waiters.remove(*index);
            })
        }
        self.delete_empty_waiters();
    }

    pub fn wake_for_timeout(&mut self) {
        let now = read_timer();
        for (_, waiters) in self.map.iter_mut() {
            let mut record = vec![];
            for (index, waiter) in waiters.iter_mut().enumerate() {
                if let Some(wait_time) = waiter.wait_time {
                    if wait_time <= now {
                        *waiter.timeout_flag.lock() = true;
                        let task = waiter.wake();
                        let mut task_manager = TASK_MANAGER.lock();
                        task_manager.push_back(task);
                        record.push(index);
                    }
                }
            }
            record.iter().for_each(|index| {
                waiters.remove(*index);
            })
        }
        // delete empty waiters
        self.delete_empty_waiters();
    }

    fn delete_empty_waiters(&mut self) {
        let mut record = vec![];
        for (futex, waiters) in self.map.iter() {
            if waiters.is_empty() {
                record.push(*futex);
            }
        }
        record.iter().for_each(|futex| {
            self.map.remove(futex);
        })
    }

    pub fn wake(&mut self, futex: usize, num: usize) -> AlienResult<usize> {
        if let Some(waiters) = self.map.get_mut(&futex) {
            error!("there are {} waiters, wake {}", waiters.len(), num);
            let min_index = min(num, waiters.len());
            for i in 0..min_index {
                let task = waiters[i].wake();
                let mut task_manager = TASK_MANAGER.lock();
                task_manager.push_back(task);
            }
            // delete waiters
            waiters.drain(0..min_index);
            warn!("wake {} tasks", min_index);
            Ok(min_index)
        } else {
            error!("futex {} not found", futex);
            Err(AlienError::Other)
        }
    }

    pub fn requeue(
        &mut self,
        requeue_futex: usize,
        num: usize,
        old_futex: usize,
    ) -> AlienResult<usize> {
        if num == 0 {
            return Ok(0);
        }
        // move waiters
        let mut waiters = self.map.remove(&old_futex).unwrap();
        // create new waiters
        let new_waiters = self.map.entry(requeue_futex).or_insert(Vec::new());
        let min_index = min(num, waiters.len());
        error!("requeue {} waiters", min_index);
        for _ in 0..min_index {
            let waiter = waiters.pop().unwrap();
            new_waiters.push(waiter);
        }
        // insert old waiters
        if !waiters.is_empty() {
            self.map.insert(old_futex, waiters);
        }
        Ok(min_index)
    }
}
