#![no_std]
#![forbid(unsafe_code)]
mod fs;
mod mm;
mod task;
mod time;

extern crate alloc;
extern crate log;

use alloc::{boxed::Box, sync::Arc};

use basic::println;
use constants::AlienResult;
use interface::{Basic, DomainType, SysCallDomain, TaskDomain, VfsDomain};

use crate::{fs::*, mm::*, task::*, time::*};

#[derive(Debug)]
struct SysCallDomainImpl {
    vfs_domain: Arc<dyn VfsDomain>,
    task_domain: Arc<dyn TaskDomain>,
}

impl SysCallDomainImpl {
    pub fn new(vfs_domain: Arc<dyn VfsDomain>, task_domain: Arc<dyn TaskDomain>) -> Self {
        Self {
            vfs_domain,
            task_domain,
        }
    }
}

impl Basic for SysCallDomainImpl {}

impl SysCallDomain for SysCallDomainImpl {
    fn init(&self) -> AlienResult<()> {
        Ok(())
    }

    fn call(&self, syscall_id: usize, args: [usize; 6]) -> AlienResult<isize> {
        let syscall_name = constants::syscall_name(syscall_id);
        // let pid = self.task_domain.current_pid().unwrap();
        // let tid = self.task_domain.current_tid().unwrap();
        // info!("[pid:{} tid:{}] syscall: {} {:?}",pid,tid, syscall_name, args);
        match syscall_id {
            29 => sys_ioctl(
                &self.vfs_domain,
                &self.task_domain,
                args[0],
                args[1],
                args[2],
            ),
            63 => sys_read(
                &self.vfs_domain,
                &self.task_domain,
                args[0],
                args[1],
                args[2],
            ),
            64 => sys_write(
                &self.vfs_domain,
                &self.task_domain,
                args[0],
                args[1] as *const u8,
                args[2],
            ),
            96 => sys_set_tid_address(&self.task_domain, args[0]),
            113 => sys_clock_gettime(&self.task_domain, args[0], args[1]),
            124 => sys_yield(&self.task_domain),
            154 => sys_set_pgid(&self.task_domain),
            155 => sys_get_pgid(&self.task_domain),
            157 => sys_set_sid(&self.task_domain),
            172 => sys_get_pid(&self.task_domain),
            173 => sys_get_ppid(&self.task_domain),
            174 => sys_getuid(&self.task_domain),
            175 => sys_get_euid(&self.task_domain),
            176 => sys_get_gid(&self.task_domain),
            177 => sys_get_egid(&self.task_domain),
            178 => sys_get_tid(&self.task_domain),
            214 => sys_brk(&self.vfs_domain, &self.task_domain, args[0]),
            220 => sys_clone(
                &self.task_domain,
                args[0],
                args[1],
                args[2],
                args[3],
                args[4],
            ),
            221 => sys_execve(&self.task_domain, args[0], args[1], args[2]),
            222 => sys_mmap(
                &self.task_domain,
                args[0],
                args[1],
                args[2],
                args[3],
                args[4],
                args[5],
            ),
            260 => sys_wait4(&self.task_domain, args[0], args[1], args[2], args[3]),

            _ => panic!("syscall [{}: {}] not found", syscall_id, syscall_name),
        }
    }
}

pub fn main() -> Box<dyn SysCallDomain> {
    let vfs_domain = basic::get_domain("vfs").unwrap();
    let vfs_domain = match vfs_domain {
        DomainType::VfsDomain(vfs_domain) => vfs_domain,
        _ => panic!("vfs domain not found"),
    };
    let task_domain = basic::get_domain("task").unwrap();
    let task_domain = match task_domain {
        DomainType::TaskDomain(task_domain) => task_domain,
        _ => panic!("task domain not found"),
    };

    println!("syscall domain began to work");
    Box::new(SysCallDomainImpl::new(vfs_domain, task_domain))
}
