#![no_std]
#![allow(unused)]

extern crate alloc;
#[macro_use]
extern crate log;

use alloc::sync::Arc;
use spin::Once;

use interface::TaskDomain;

pub mod task;
pub mod trap;

static TASK_DOMAIN: Once<Arc<dyn TaskDomain>> = Once::new();

pub fn register_task_domain(task_domain: Arc<dyn TaskDomain>) {
    TASK_DOMAIN.call_once(|| task_domain);
}

pub fn run_task() {
    let task_domain = TASK_DOMAIN.get().expect("task domain not init");
    task_domain.run();
}
