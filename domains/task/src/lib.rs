#![no_std]
// #![deny(unsafe_code)]
extern crate alloc;
extern crate libsyscall;
#[macro_use]
extern crate log;
mod elf;
mod init;
mod kthread;
mod processor;
mod resource;
mod scheduler;
mod task;
mod vfs_shim;

use crate::processor::current_task;
use crate::scheduler::run_task;
use alloc::sync::Arc;
use constants::AlienError;
use interface::{Basic, InodeId, TaskDomain, TmpHeapInfo};
use rref::{RRef, RpcError, RpcResult};

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

    fn trap_frame_virt_addr(&self) -> RpcResult<usize> {
        Ok(processor::current_trap_frame_ptr())
    }
    fn current_task_satp(&self) -> RpcResult<usize> {
        Ok(processor::current_user_token())
    }

    fn trap_frame_phy_addr(&self) -> RpcResult<usize> {
        Ok(processor::current_trap_frame() as *mut _ as usize)
    }

    fn heap_info(&self, mut tmp_heap_info: RRef<TmpHeapInfo>) -> RpcResult<RRef<TmpHeapInfo>> {
        let task = current_task().unwrap();
        let inner = task.inner.lock();
        let guard = inner.heap.lock();
        *tmp_heap_info = TmpHeapInfo {
            start: guard.start,
            current: guard.current,
        };
        Ok(tmp_heap_info)
    }

    fn brk(&self, addr: usize) -> RpcResult<isize> {
        let task = current_task().unwrap();
        let new_addr = task.extend_heap(addr);
        Ok(new_addr as isize)
    }

    fn get_fd(&self, fd: usize) -> RpcResult<InodeId> {
        let task = current_task().unwrap();
        let file = task
            .get_file(fd)
            .ok_or(RpcError::Alien(AlienError::EBADF))?;
        Ok(file.inode_id())
    }

    fn copy_to_user(&self, src: *const u8, dst: *mut u8, len: usize) -> RpcResult<()> {
        let task = current_task().unwrap();
        task.copy_to_user(src, dst, len);
        Ok(())
    }

    fn copy_from_user(&self, src: *const u8, dst: *mut u8, len: usize) -> RpcResult<()> {
        let task = current_task().unwrap();
        task.copy_from_user(src, dst, len);
        Ok(())
    }
}

pub fn main() -> Arc<dyn TaskDomain> {
    let vfs_domain = libsyscall::get_vfs_domain().unwrap();
    vfs_shim::init_vfs_domain(vfs_domain);
    init::init_task();
    Arc::new(TaskDomainImpl::new())
}
