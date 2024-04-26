use constants::AlienResult;
use downcast_rs::{impl_downcast, DowncastSync};
use gproxy::proxy;
use rref::RRef;
use task_meta::TaskMeta;

use crate::Basic;

#[proxy(SchedulerDomainProxy)]
pub trait SchedulerDomain: Basic + DowncastSync {
    fn init(&self) -> AlienResult<()>;
    fn run(&self) -> AlienResult<()>;
    fn current_tid(&self) -> AlienResult<Option<usize>>;
    /// return kstack top
    fn add_one_task(&self, task_meta: RRef<TaskMeta>) -> AlienResult<usize>;
    /// Set current task to wait and switch to next task
    fn current_to_wait(&self) -> AlienResult<()>;
    /// Wake up the task with tid
    fn wake_up_wait_task(&self, tid: usize) -> AlienResult<()>;
    /// Yield the current task
    fn yield_now(&self) -> AlienResult<()>;
    fn exit_now(&self) -> AlienResult<()>;
}

impl_downcast!(sync SchedulerDomain);
