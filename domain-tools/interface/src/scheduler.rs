use alloc::vec::Vec;

use constants::AlienResult;
use downcast_rs::{impl_downcast, DowncastSync};
use gproxy::proxy;
use rref::RRef;
use task_meta::{TaskContext, TaskMeta};

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
    fn dump_meta_data(&self, data: &mut SchedulerDataContainer) -> AlienResult<()>;
}

impl_downcast!(sync SchedulerDomain);

#[derive(Debug, Copy, Clone, Default)]
pub struct CpuLocalData {
    pub cpu_context: TaskContext,
    pub task: Option<TaskData>,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct KStackData {
    pub kstack_top: usize,
    pub pages: usize,
}
#[derive(Debug, Copy, Clone)]
pub struct TaskData {
    pub task_meta: TaskMeta,
    pub kstack_data: KStackData,
}

#[derive(Debug, Default)]
pub struct SchedulerDataContainer {
    pub cpu_local: CpuLocalData,
    pub task_wait_queue: Vec<TaskData>,
    pub task_ready_queue: Vec<TaskData>,
}
