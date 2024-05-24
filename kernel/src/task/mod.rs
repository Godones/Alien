mod processor;
mod resource;
mod scheduler;

use alloc::sync::Arc;
use core::arch::global_asm;

use basic::task::TaskContext;
use interface::{SchedulerDomain, TaskDomain};
use ksync::Mutex;
pub use processor::current_tid;
pub use scheduler::{exit_now, remove_task, wait_now, wake_up_wait_task, yield_now};
use spin::Once;
use task_meta::TaskMeta;

use crate::{error::AlienResult, task::resource::TaskMetaExt};

global_asm!(include_str!("switch.asm"));

extern "C" {
    fn __switch(now: *mut TaskContext, next: *const TaskContext);
}

/// 交换前后两个线程的上下文，调用 `switch.asm` 中的 `__switch`
#[inline(always)]
pub fn switch(now: *mut TaskContext, next: *const TaskContext) {
    unsafe {
        __switch(now, next);
    }
}

pub static TASK_DOMAIN: Once<Arc<dyn TaskDomain>> = Once::new();
#[macro_export]
macro_rules! task_domain {
    () => {
        crate::task::TASK_DOMAIN
            .get()
            .expect("task domain not init")
    };
}

pub fn register_scheduler_domain(scheduler_domain: Arc<dyn SchedulerDomain>) {
    scheduler::set_scheduler(scheduler_domain);
}

pub fn register_task_domain(task_domain: Arc<dyn TaskDomain>) {
    TASK_DOMAIN.call_once(|| task_domain);
}

pub fn run_task() {
    processor::schedule();
}

pub fn add_one_task(task_meta: TaskMeta) -> AlienResult<usize> {
    let task_meta_ext = TaskMetaExt::new(task_meta);
    let kstack_top = task_meta_ext.kstack.top();
    scheduler::add_task(Arc::new(Mutex::new(task_meta_ext)));
    Ok(kstack_top.as_usize())
}
