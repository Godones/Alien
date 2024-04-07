#![no_std]
#![allow(unused)]

extern crate alloc;
#[macro_use]
extern crate log;

use alloc::sync::Arc;

use interface::{PLICDomain, SysCallDomain, TaskDomain};
use spin::Once;

pub mod trap;

static TASK_DOMAIN: Once<Arc<dyn TaskDomain>> = Once::new();
static SYSCALL_DOMAIN: Once<Arc<dyn SysCallDomain>> = Once::new();
static PLIC_DOMAIN: Once<Arc<dyn PLICDomain>> = Once::new();
pub fn register_task_domain(task_domain: Arc<dyn TaskDomain>) {
    TASK_DOMAIN.call_once(|| task_domain);
}

pub fn register_syscall_domain(syscall_domain: Arc<dyn SysCallDomain>) {
    SYSCALL_DOMAIN.call_once(|| syscall_domain);
}

pub fn register_plic_domain(plic_domain: Arc<dyn PLICDomain>) {
    PLIC_DOMAIN.call_once(|| plic_domain);
}

pub fn run_task() {
    let task_domain = TASK_DOMAIN.get().expect("task domain not init");
    task_domain.run();
}
