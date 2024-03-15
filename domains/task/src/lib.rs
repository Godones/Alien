#![no_std]
// #![deny(unsafe_code)]
extern crate alloc;
extern crate libsyscall;
#[macro_use]
extern crate log;
mod elf;
mod init;
mod kstack;
mod kthread;
mod processor;
mod resource;
mod scheduler;
mod task;
mod vfs_shim;

use crate::scheduler::run_task;
use alloc::sync::Arc;
use interface::{Basic, TaskDomain};

#[derive(Debug)]
pub struct TaskDomainImpl {}

impl TaskDomainImpl {
    pub fn new() -> Self {
        Self {}
    }
}

impl Basic for TaskDomainImpl {}

impl TaskDomain for TaskDomainImpl {
    fn run(&self) {
        run_task()
    }

    fn current_task_trap_frame_ptr(&self) -> usize {
        processor::current_trap_frame() as *mut _ as usize
    }
    fn current_task_satp(&self) -> usize {
        processor::current_user_token()
    }
}

pub fn main() -> Arc<dyn TaskDomain> {
    let vfs_domain = libsyscall::get_vfs_domain().unwrap();
    vfs_shim::init_vfs_domain(vfs_domain);
    init::init_task();
    info!("task domain start...");
    Arc::new(TaskDomainImpl::new())
}
