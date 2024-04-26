use alloc::{collections::VecDeque, sync::Arc};

use basic::sync::Mutex;
use common_scheduler::{resource::TaskMetaExt, Scheduler};

#[derive(Debug)]
pub struct FiFoScheduler {
    tasks: VecDeque<Arc<Mutex<TaskMetaExt>>>,
}

impl FiFoScheduler {
    pub const fn new() -> Self {
        Self {
            tasks: VecDeque::new(),
        }
    }
}

impl Scheduler for FiFoScheduler {
    fn add_task(&mut self, task_meta: Arc<Mutex<TaskMetaExt>>) {
        self.tasks.push_back(task_meta);
    }

    fn fetch_task(&mut self) -> Option<Arc<Mutex<TaskMetaExt>>> {
        self.tasks.pop_front()
    }

    fn name(&self) -> &'static str {
        "FiFoScheduler"
    }
}
