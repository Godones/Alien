#![no_std]

mod fs;
mod mm;

extern crate alloc;
#[macro_use]
extern crate log;

use crate::fs::sys_write;
use crate::mm::sys_brk;
use alloc::sync::Arc;
use basic::println;
use constants::AlienResult;
use interface::{Basic, DomainType, SysCallDomain, TaskDomain, VfsDomain};

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
        info!("syscall: {} {:?}", syscall_name, args);
        match syscall_id {
            64 => sys_write(
                &self.vfs_domain,
                &self.task_domain,
                args[0],
                args[1] as *const u8,
                args[2],
            )
            .map_err(|e| e.into()),
            214 => sys_brk(&self.vfs_domain, &self.task_domain, args[0]).map_err(|e| e.into()),
            _ => panic!("syscall not found"),
        }
    }
}

pub fn main() -> Arc<dyn SysCallDomain> {
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
    Arc::new(SysCallDomainImpl::new(vfs_domain, task_domain))
}
