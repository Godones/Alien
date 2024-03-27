#![no_std]
// #![forbid(unsafe_code)]
extern crate alloc;
#[macro_use]
extern crate log;
mod elf;
mod init;
mod kthread;
mod processor;
mod resource;
mod scheduler;
mod syscall;
mod task;
mod vfs_shim;
mod wait_queue;

use crate::processor::current_task;
use crate::scheduler::{do_suspend, run_task};
use alloc::sync::Arc;
use constants::AlienError;
use constants::AlienResult;
use interface::{Basic, DomainType, InodeId, TaskDomain, TmpHeapInfo};
use rref::RRef;

#[derive(Debug)]
pub struct TaskDomainImpl {}

impl TaskDomainImpl {
    pub fn new() -> Self {
        Self {}
    }
}

impl Basic for TaskDomainImpl {}

impl TaskDomain for TaskDomainImpl {
    fn init(&self) -> AlienResult<()> {
        let vfs_domain = basic::get_domain("vfs").unwrap();
        let vfs_domain = match vfs_domain {
            DomainType::VfsDomain(vfs_domain) => vfs_domain,
            _ => panic!("vfs domain not found"),
        };
        vfs_shim::init_vfs_domain(vfs_domain);
        init::init_task();
        Ok(())
    }
    fn run(&self) {
        run_task()
    }

    fn trap_frame_virt_addr(&self) -> AlienResult<usize> {
        Ok(processor::current_trap_frame_ptr())
    }
    fn current_task_satp(&self) -> AlienResult<usize> {
        Ok(processor::current_user_token())
    }

    fn trap_frame_phy_addr(&self) -> AlienResult<usize> {
        Ok(processor::current_trap_frame() as *mut _ as usize)
    }

    fn heap_info(&self, mut tmp_heap_info: RRef<TmpHeapInfo>) -> AlienResult<RRef<TmpHeapInfo>> {
        let task = current_task().unwrap();
        let guard = task.heap.lock();
        *tmp_heap_info = TmpHeapInfo {
            start: guard.start,
            current: guard.current,
        };
        Ok(tmp_heap_info)
    }

    fn get_fd(&self, fd: usize) -> AlienResult<InodeId> {
        let task = current_task().unwrap();
        let file = task.get_file(fd).ok_or(AlienError::EBADF)?;
        Ok(file.inode_id())
    }

    fn copy_to_user(&self, src: *const u8, dst: *mut u8, len: usize) -> AlienResult<()> {
        let task = current_task().unwrap();
        task.copy_to_user(src, dst, len);
        Ok(())
    }

    fn copy_from_user(&self, src: *const u8, dst: *mut u8, len: usize) -> AlienResult<()> {
        let task = current_task().unwrap();
        task.copy_from_user(src, dst, len);
        Ok(())
    }

    fn current_tid(&self) -> AlienResult<usize> {
        let task = current_task().unwrap();
        Ok(task.tid.raw())
    }

    fn current_pid(&self) -> AlienResult<usize> {
        let task = current_task().unwrap();
        Ok(task.pid.raw())
    }
    fn current_ppid(&self) -> AlienResult<usize> {
        Ok(0)
    }
    fn current_to_wait(&self) -> AlienResult<()> {
        wait_queue::current_to_wait();
        Ok(())
    }
    fn wake_up_wait_task(&self, tid: usize) -> AlienResult<()> {
        wait_queue::wake_up_wait_task(tid);
        Ok(())
    }

    fn do_brk(&self, addr: usize) -> AlienResult<isize> {
        let task = current_task().unwrap();
        let new_addr = task.extend_heap(addr);
        Ok(new_addr as isize)
    }

    fn do_clone(
        &self,
        flags: usize,
        stack: usize,
        ptid: usize,
        tls: usize,
        ctid: usize,
    ) -> AlienResult<isize> {
        syscall::clone::do_clone(flags, stack, ptid, tls, ctid)
    }

    fn do_wait4(
        &self,
        pid: isize,
        exit_code_ptr: usize,
        options: u32,
        _rusage: usize,
    ) -> AlienResult<isize> {
        syscall::wait::do_wait4(pid, exit_code_ptr, options, _rusage)
    }

    fn do_execve(
        &self,
        filename_ptr: usize,
        argv_ptr: usize,
        envp_ptr: usize,
    ) -> AlienResult<isize> {
        syscall::execve::do_execve(filename_ptr, argv_ptr, envp_ptr)
    }

    fn do_yield(&self) -> AlienResult<isize> {
        do_suspend();
        Ok(0)
    }
}

pub fn main() -> Arc<dyn TaskDomain> {
    Arc::new(TaskDomainImpl::new())
}
