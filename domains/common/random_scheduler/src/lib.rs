#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;
use alloc::{boxed::Box, collections::VecDeque, sync::Arc};

use basic::sync::Mutex;
use common_scheduler::{resource::TaskMetaExt, CommonSchedulerDomain, Scheduler};
use interface::SchedulerDomain;

#[derive(Debug)]
pub struct RandomScheduler {
    fetch_mask: bool,
    tasks: VecDeque<Arc<Mutex<TaskMetaExt>>>,
}

impl RandomScheduler {
    pub const fn new() -> Self {
        Self {
            fetch_mask: false,
            tasks: VecDeque::new(),
        }
    }
}

impl Scheduler for RandomScheduler {
    fn add_task(&mut self, task_meta: Arc<Mutex<TaskMetaExt>>) {
        self.tasks.push_back(task_meta);
    }

    fn fetch_task(&mut self) -> Option<Arc<Mutex<TaskMetaExt>>> {
        if self.fetch_mask {
            self.fetch_mask = false;
            self.tasks.pop_front()
        } else {
            self.fetch_mask = true;
            self.tasks.pop_back()
        }
    }

    fn name(&self) -> &'static str {
        "RandomScheduler"
    }
}

pub fn main() -> Box<dyn SchedulerDomain> {
    Box::new(CommonSchedulerDomain::new(Box::new(RandomScheduler::new())))
}
