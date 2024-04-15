#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;
#[macro_use]
extern crate log;
mod elf;
mod init;
mod kthread;
mod processor;
mod resource;
mod syscall;
mod task;
mod vfs_shim;

use alloc::{boxed::Box, sync::Arc};

use basic::println;
use constants::{AlienError, AlienResult};
use interface::{Basic, DomainType, InodeID, SchedulerDomain, TaskDomain, TmpHeapInfo};
use memory_addr::VirtAddr;
use rref::RRef;
use spin::Once;

use crate::processor::current_task;

pub static SCHEDULER_DOMAIN: Once<Arc<dyn SchedulerDomain>> = Once::new();

#[macro_export]
macro_rules! scheduler_domain {
    () => {
        crate::SCHEDULER_DOMAIN
            .get()
            .expect("scheduler domain not found")
    };
}

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

        let scheduler_domain = basic::get_domain("scheduler").unwrap();
        let scheduler_domain = match scheduler_domain {
            DomainType::SchedulerDomain(scheduler_domain) => scheduler_domain,
            _ => panic!("scheduler domain not found"),
        };
        SCHEDULER_DOMAIN.call_once(|| scheduler_domain);

        init::init_task();
        println!("Init task domain success");
        Ok(())
    }

    fn trap_frame_virt_addr(&self) -> AlienResult<usize> {
        let task = current_task().unwrap();
        let addr = task.trap_frame_virt_ptr();
        Ok(addr.as_usize())
    }

    fn current_task_satp(&self) -> AlienResult<usize> {
        let task = current_task().unwrap();
        Ok(task.token())
    }

    fn trap_frame_phy_addr(&self) -> AlienResult<usize> {
        let task = current_task().unwrap();
        Ok(task.trap_frame_phy_ptr().as_usize())
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

    fn get_fd(&self, fd: usize) -> AlienResult<InodeID> {
        let task = current_task().unwrap();
        let file = task.get_file(fd).ok_or(AlienError::EBADF)?;
        Ok(file.inode_id())
    }

    fn copy_to_user(&self, dst: usize, buf: &[u8]) -> AlienResult<()> {
        let task = current_task().unwrap();
        task.write_bytes_to_user(VirtAddr::from(dst), buf)
    }

    fn copy_from_user(&self, src: usize, buf: &mut [u8]) -> AlienResult<()> {
        let task = current_task().unwrap();
        task.read_bytes_from_user(VirtAddr::from(src), buf)
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
        let task = current_task().unwrap();
        let p = task.inner().parent.clone();
        if p.is_none() {
            Ok(0)
        } else {
            let p = p.unwrap().upgrade().unwrap();
            Ok(p.pid() as _)
        }
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
        syscall::execve::do_execve(
            VirtAddr::from(filename_ptr),
            argv_ptr.into(),
            envp_ptr.into(),
        )
    }

    fn do_set_tid_address(&self, tidptr: usize) -> AlienResult<isize> {
        let task = current_task().unwrap();
        task.set_tid_address(tidptr);
        Ok(task.tid() as _)
    }
    fn do_mmap(
        &self,
        start: usize,
        len: usize,
        prot: u32,
        flags: u32,
        fd: usize,
        offset: usize,
    ) -> AlienResult<isize> {
        syscall::mmap::do_mmap(start, len, prot, flags, fd, offset)
    }
}

pub fn main() -> Box<dyn TaskDomain> {
    Box::new(TaskDomainImpl::new())
}
