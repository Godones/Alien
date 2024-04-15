use constants::AlienResult;
use gproxy::proxy;
use rref::RRef;
use task_meta::TaskMeta;

use crate::Basic;

#[proxy(SchedulerDomainProxy)]
pub trait SchedulerDomain: Basic {
    fn init(&self) -> AlienResult<()>;
    fn run(&self) -> AlienResult<()>;
    fn current_tid(&self) -> AlienResult<Option<usize>>;
    fn add_one_task(&self, task_meta: RRef<TaskMeta>) -> AlienResult<()>;
    /// Set current task to wait and switch to next task
    fn current_to_wait(&self) -> AlienResult<()>;
    /// Wake up the task with tid
    fn wake_up_wait_task(&self, tid: usize) -> AlienResult<()>;
    /// Yield the current task
    fn yield_now(&self) -> AlienResult<()>;
}
