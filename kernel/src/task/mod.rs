pub mod continuation;
mod processor;
mod resource;
mod scheduler;

use alloc::sync::Arc;
use core::arch::global_asm;

use arch::hart_id;
use basic::task::TaskContext;
use config::CPU_NUM;
use interface::{SchedulerDomain, TaskDomain};
use ksync::Mutex;
pub use processor::current_tid;
pub use scheduler::{
    exit_now, get_task_priority, is_task_exit, remove_task, set_task_priority, wait_now,
    wake_up_wait_task, yield_now,
};
use spin::Once;
use task_meta::{TaskMeta, TaskStatus};

use crate::{
    error::AlienResult,
    task::{processor::current_task, resource::TaskMetaExt, scheduler::TASK_WAIT_QUEUE},
};

global_asm!(include_str!("switch.asm"));

extern "C" {
    fn __switch(now: *mut TaskContext, next: *const TaskContext);
}

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
    processor::cpu_loop();
}

pub fn add_one_task(task_meta: TaskMeta, is_kthread: bool) -> AlienResult<usize> {
    let mut task_meta_ext = TaskMetaExt::new(task_meta, is_kthread);
    let kstack_top = task_meta_ext.kstack.top();

    task_meta_ext.set_status(TaskStatus::Waiting);
    let tid = task_meta_ext.tid();
    let task = Arc::new(Mutex::new(task_meta_ext));
    TASK_WAIT_QUEUE.lock().insert(tid, task);

    Ok(kstack_top.as_usize())
}

pub fn synchronize_rcu() {
    let task = current_task();
    if task.is_none() {
        return;
    }
    let task = task.expect("no current task");
    let mut guard = task.lock();
    let old_cpus_allowed = guard.scheduling_info.as_ref().unwrap().cpus_allowed;
    guard.scheduling_info.as_mut().unwrap().cpus_allowed = (1 << CPU_NUM) - 1;
    println!("set cpus_allowed to {}", (1 << CPU_NUM) - 1);
    drop(guard);
    loop {
        let mut guard = task.lock();
        let cpu_id = hart_id();
        let mut cpus_allowed = guard.scheduling_info.as_ref().unwrap().cpus_allowed;
        cpus_allowed &= !(1 << cpu_id);
        if cpus_allowed == CPU_OK {
            println!("synchronize_rcu done");
            guard.scheduling_info.as_mut().unwrap().cpus_allowed = old_cpus_allowed;
            break;
        }
        guard.scheduling_info.as_mut().unwrap().cpus_allowed = cpus_allowed;
        println!("synchronize_rcu cpus_allowed: {}", cpus_allowed);
        drop(guard);
        yield_now();
    }
}

#[cfg(vf2)]
const CPU_OK: usize = 1;

#[cfg(not(vf2))]
const CPU_OK: usize = 0;
