#![no_std]
#![forbid(unsafe_code)]

mod processor;
mod scheduler;

extern crate alloc;

use alloc::{boxed::Box, sync::Arc};

use basic::{println, sync::Mutex};
use constants::AlienResult;
use interface::{Basic, SchedulerDomain};
use rref::RRef;
pub use scheduler::Scheduler;
use task_meta::TaskMeta;

use crate::processor::run_task;

#[derive(Debug)]
pub struct CommonSchedulerDomain {
    name: &'static str,
}

impl CommonSchedulerDomain {
    pub fn new(global_scheduler: Box<dyn Scheduler>) -> Self {
        let name = global_scheduler.name();
        scheduler::set_scheduler(global_scheduler);
        Self { name }
    }
}

impl Basic for CommonSchedulerDomain {}

impl SchedulerDomain for CommonSchedulerDomain {
    fn init(&self) -> AlienResult<()> {
        println!("SchedulerDomain init, name: {}", self.name);
        Ok(())
    }
    fn run(&self) -> AlienResult<()> {
        run_task();
    }

    fn current_tid(&self) -> AlienResult<Option<usize>> {
        let task = processor::current_task();
        match task {
            None => Ok(None),
            Some(task) => {
                let tid = task.lock().tid();
                Ok(Some(tid))
            }
        }
    }

    fn add_one_task(&self, task_meta: RRef<TaskMeta>) -> AlienResult<()> {
        scheduler::add_task(Arc::new(Mutex::new(*task_meta)));
        Ok(())
    }
    fn current_to_wait(&self) -> AlienResult<()> {
        scheduler::current_to_wait();
        Ok(())
    }

    fn wake_up_wait_task(&self, tid: usize) -> AlienResult<()> {
        scheduler::wake_up_wait_task(tid);
        Ok(())
    }
    fn yield_now(&self) -> AlienResult<()> {
        scheduler::do_suspend();
        Ok(())
    }
}
