#![no_std]
#![forbid(unsafe_code)]
mod domain;
mod fs;
mod mm;
mod signal;
mod socket;
mod system;
mod task;
mod time;

extern crate alloc;
extern crate log;

use alloc::{boxed::Box, sync::Arc};

use basic::println;
use constants::AlienResult;
use interface::{Basic, DomainType, SchedulerDomain, SysCallDomain, TaskDomain, VfsDomain};

use crate::{domain::*, fs::*, mm::*, signal::*, socket::sys_socket, system::*, task::*, time::*};

#[derive(Debug)]
struct SysCallDomainImpl {
    vfs_domain: Arc<dyn VfsDomain>,
    task_domain: Arc<dyn TaskDomain>,
    scheduler: Arc<dyn SchedulerDomain>,
}

impl SysCallDomainImpl {
    pub fn new(
        vfs_domain: Arc<dyn VfsDomain>,
        task_domain: Arc<dyn TaskDomain>,
        scheduler: Arc<dyn SchedulerDomain>,
    ) -> Self {
        Self {
            vfs_domain,
            task_domain,
            scheduler,
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
            23 => sys_dup(&self.task_domain, args[0]),
            24 => sys_dup2(&self.task_domain, args[0], args[1]),
            25 => sys_fcntl(
                &self.vfs_domain,
                &self.task_domain,
                args[0],
                args[1],
                args[2],
            ),
            29 => sys_ioctl(
                &self.vfs_domain,
                &self.task_domain,
                args[0],
                args[1],
                args[2],
            ),
            48 => sys_faccessat(
                &self.vfs_domain,
                &self.task_domain,
                args[0],
                args[1],
                args[2],
                args[3],
            ),
            56 => sys_openat(
                &self.vfs_domain,
                &self.task_domain,
                args[0],
                args[1] as *const u8,
                args[2],
                args[3],
            ),
            57 => sys_close(&self.vfs_domain, &self.task_domain, args[0]),
            59 => sys_pipe2(&self.task_domain, &self.vfs_domain, args[0], args[1]),
            61 => sys_getdents64(
                &self.vfs_domain,
                &self.task_domain,
                args[0],
                args[1],
                args[2],
            ),
            62 => sys_lseek(
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
            65 => sys_readv(
                &self.vfs_domain,
                &self.task_domain,
                args[0],
                args[1],
                args[2],
            ),
            66 => sys_writev(
                &self.vfs_domain,
                &self.task_domain,
                args[0],
                args[1],
                args[2],
            ),
            72 => sys_pselect6(
                &self.vfs_domain,
                &self.task_domain,
                &self.scheduler,
                args[0],
                args[1],
                args[2],
                args[3],
                args[4],
                args[5],
            ),
            79 => sys_fstatat(
                &self.vfs_domain,
                &self.task_domain,
                args[0],
                args[1] as *const u8,
                args[2],
                args[3],
            ),
            80 => sys_fstat(&self.vfs_domain, &self.task_domain, args[0], args[1]),
            93 => sys_exit(&self.task_domain, args[0]),
            94 => sys_exit_group(&self.task_domain, args[0]),
            96 => sys_set_tid_address(&self.task_domain, args[0]),
            113 => sys_clock_gettime(&self.task_domain, args[0], args[1]),
            124 => sys_yield(&self.scheduler),
            134 => sys_sigaction(&self.task_domain, args[0], args[1], args[2]),
            135 => sys_sigprocmask(&self.task_domain, args[0], args[1], args[2], args[3]),
            154 => sys_set_pgid(&self.task_domain),
            155 => sys_get_pgid(&self.task_domain),
            157 => sys_set_sid(&self.task_domain),
            160 => sys_uname(&self.task_domain, args[0]),
            172 => sys_get_pid(&self.task_domain),
            173 => sys_get_ppid(&self.task_domain),
            174 => sys_getuid(&self.task_domain),
            175 => sys_get_euid(&self.task_domain),
            176 => sys_get_gid(&self.task_domain),
            177 => sys_get_egid(&self.task_domain),
            178 => sys_get_tid(&self.task_domain),
            198 => sys_socket(&self.task_domain, args[0], args[1], args[2]),
            214 => sys_brk(&self.vfs_domain, &self.task_domain, args[0]),
            215 => sys_unmap(&self.task_domain, args[0], args[1]),
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
            261 => sys_prlimit64(&self.task_domain, args[0], args[1], args[2], args[3]),
            888 => sys_load_domain(
                &self.task_domain,
                &self.vfs_domain,
                args[0],
                args[1] as u8,
                args[2],
                args[3],
            ),
            889 => sys_replace_domain(&self.task_domain, args[0], args[1], args[2], args[3]),
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

    let scheduler_domain = basic::get_domain("scheduler").unwrap();
    let scheduler_domain = match scheduler_domain {
        DomainType::SchedulerDomain(scheduler_domain) => scheduler_domain,
        _ => panic!("scheduler domain not found"),
    };

    println!("syscall domain began to work");
    Box::new(SysCallDomainImpl::new(
        vfs_domain,
        task_domain,
        scheduler_domain,
    ))
}
