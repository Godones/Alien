#![no_std]

mod fs;
mod mm;

extern crate alloc;
#[macro_use]
extern crate log;

use crate::fs::sys_write;
use crate::mm::sys_brk;
use alloc::sync::Arc;
use interface::{Basic, SysCallDomain, TaskDomain, VfsDomain};
use libsyscall::println;
use rref::RpcResult;

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
    fn call(&self, syscall_id: usize, args: [usize; 6]) -> RpcResult<isize> {
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
    let vfs_domain = libsyscall::get_vfs_domain().unwrap();
    let task_domain = libsyscall::get_task_domain().unwrap();
    println!("syscall domain began to work");
    Arc::new(SysCallDomainImpl::new(vfs_domain, task_domain))
}
