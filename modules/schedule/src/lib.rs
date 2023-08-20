//! A simple scheduler for smp
//!
//! Every hart has a task queue, and the scheduler will fetch a task from the queue.
//! If the queue is empty, the scheduler will fetch a task from the global queue.
//! When task is suspended, it will be pushed into the local queue and if the local queue is full,
//! it will be pushed into the global queue.
//!
//! # Example
//! ```
//! use std::sync::atomic::{AtomicUsize, Ordering};use crate::schedule::*;
//! static TMP: AtomicUsize = AtomicUsize::new(0);
//! #[derive(Debug)]
//! struct ScheduleHartImpl;
//!
//! impl ScheduleHart for ScheduleHartImpl {
//!     fn hart_id() -> usize {
//!         TMP.load(Ordering::SeqCst)
//!     }
//! }
//! let s = Scheduler::<u32, ScheduleHartImpl, 4, 16>::new();
//! s.push_to_global(1); // push to global queue
//! s.push(2).unwrap(); // push to local queue 0
//! TMP.store(1, Ordering::SeqCst);
//! s.push(3).unwrap(); // push to local queue 1
//!  // println!("{:#?}", s);
//! let val1 = s.fetch();
//! assert_eq!(val1, Some(3));
//! TMP.store(0, Ordering::SeqCst);
//! let val2 = s.fetch();
//! assert_eq!(val2, Some(2));
//!```
//!

#![cfg_attr(not(test), no_std)]
#![deny(missing_docs)]
extern crate alloc;

use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::fmt::Debug;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering};

/// The max size of global task queue
pub const GLOBAL_TASK_MAX: usize = 65536;

/// The trait of schedule
pub trait Schedule {
    /// The type of task
    type Task: Clone + Debug;
    /// fetch a task from the queue
    fn fetch(&self) -> Option<Self::Task>;
    /// push a task into the queue
    fn push(&self, task: Self::Task) -> Result<(), Self::Task>;
}

/// The trait for getting hart id
pub trait ScheduleHart: Debug {
    /// get the hart id
    fn hart_id() -> usize;
}

/// The task queue
#[derive(Debug)]
struct TaskQueue<T: Clone + Debug, const QS: usize> {
    max_size: usize,
    tasks: RefCell<VecDeque<T>>,
}

impl<T: Clone + Debug, const QS: usize> TaskQueue<T, QS> {
    pub fn new() -> Self {
        Self {
            max_size: QS,
            tasks: RefCell::new(VecDeque::new()),
        }
    }
}

impl<T: Clone + Debug, const QS: usize> Schedule for TaskQueue<T, QS> {
    type Task = T;

    fn fetch(&self) -> Option<Self::Task> {
        let mut tasks = self.tasks.borrow_mut();
        tasks.pop_front()
    }

    fn push(&self, task: Self::Task) -> Result<(), Self::Task> {
        let mut tasks = self.tasks.borrow_mut();
        if tasks.len() >= self.max_size {
            return Err(task);
        }
        tasks.push_back(task);
        Ok(())
    }
}

/// The scheduler
#[derive(Debug)]
pub struct Scheduler<T: Clone + Debug, H: ScheduleHart, const SMP: usize, const QS: usize> {
    global_lock: AtomicBool,
    global_queue: TaskQueue<T, GLOBAL_TASK_MAX>,
    local_queues: Vec<TaskQueue<T, QS>>,
    hart: PhantomData<H>,
}

impl<T: Clone + Debug, H: ScheduleHart, const SMP: usize, const QS: usize>
    Scheduler<T, H, SMP, QS>
{
    /// create a scheduler
    pub fn new() -> Self {
        let mut t = Self {
            global_lock: AtomicBool::new(false),
            global_queue: TaskQueue::new(),
            local_queues: Vec::new(),
            hart: PhantomData,
        };
        for _ in 0..SMP {
            t.local_queues.push(TaskQueue::new());
        }
        t
    }
    /// push a task into the global queue
    ///
    /// During the kernel initialization phase, the created tasks will be placed on the global queue
    pub fn push_to_global(&self, task: T) {
        while self
            .global_lock
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            // spin
        }
        self.global_queue.push(task).unwrap();
        self.global_lock.store(false, Ordering::SeqCst);
    }
}

impl<T: Clone + Debug, H: ScheduleHart, const SMP: usize, const QS: usize> Schedule
    for Scheduler<T, H, SMP, QS>
{
    type Task = T;

    fn fetch(&self) -> Option<Self::Task> {
        let hart_id = H::hart_id();
        let local_queue = &self.local_queues[hart_id];
        let task = local_queue.fetch();
        if task.is_some() {
            return task;
        }
        while self
            .global_lock
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            // spin
        }
        let task = self.global_queue.fetch();
        self.global_lock.store(false, Ordering::SeqCst);
        task
    }

    fn push(&self, task: Self::Task) -> Result<(), Self::Task> {
        let hart_id = H::hart_id();
        let local_queue = &self.local_queues[hart_id];
        let task = local_queue.push(task);
        if task.is_ok() {
            return Ok(());
        }
        while self
            .global_lock
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            // spin
        }
        self.global_queue.push(task.err().unwrap()).unwrap();
        self.global_lock.store(false, Ordering::SeqCst);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use core::sync::atomic::{AtomicUsize, Ordering};

    use crate::{Schedule, ScheduleHart, Scheduler};

    static TMP: AtomicUsize = AtomicUsize::new(0);

    #[derive(Debug)]
    struct ScheduleHartImpl;

    impl ScheduleHart for ScheduleHartImpl {
        fn hart_id() -> usize {
            TMP.load(Ordering::SeqCst)
        }
    }

    #[test]
    fn fetch_push() {
        let s = Scheduler::<u32, ScheduleHartImpl, 4, 16>::new();
        s.push_to_global(1); // push to global queue
        s.push(2).unwrap(); // push to local queue 0
        TMP.store(1, Ordering::SeqCst);
        s.push(3).unwrap(); // push to local queue 1
                            // println!("{:#?}", s);
        let val1 = s.fetch();
        assert_eq!(val1, Some(3));
        TMP.store(0, Ordering::SeqCst);
        let val2 = s.fetch();
        assert_eq!(val2, Some(2));
    }
}
