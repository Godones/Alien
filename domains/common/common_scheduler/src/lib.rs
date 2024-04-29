#![no_std]
#![forbid(unsafe_code)]

mod dump;
mod processor;
pub mod resource;
mod scheduler;

extern crate alloc;

use alloc::{boxed::Box, sync::Arc};

use basic::{println, sync::Mutex};
use constants::AlienResult;
use interface::{Basic, SchedulerDataContainer, SchedulerDomain};
use log::debug;
use rref::RRef;
pub use scheduler::Scheduler;
use task_meta::TaskMeta;

use crate::{processor::run_task, resource::TaskMetaExt};

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

    fn add_one_task(&self, task_meta: RRef<TaskMeta>) -> AlienResult<usize> {
        let task_meta = TaskMetaExt::new(*task_meta);
        let kstack_top = task_meta.kstack.top();
        scheduler::add_task(Arc::new(Mutex::new(task_meta)));
        Ok(kstack_top.as_usize())
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

    fn exit_now(&self) -> AlienResult<()> {
        debug!("<exit_now>");
        scheduler::exit_now();
        Ok(())
    }
    fn dump_meta_data(&self, data: &mut SchedulerDataContainer) -> AlienResult<()> {
        dump::dump_meta_data(data);
        Ok(())
    }
}
