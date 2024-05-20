mod ipc;
mod resource;

use alloc::sync::Arc;

use basic::AlienResult;
use interface::{SchedulerDomain, TaskDomain};
pub use ipc::*;
use log::info;
pub use resource::*;

pub fn sys_clone(
    task_domain: &Arc<dyn TaskDomain>,
    flag: usize,
    stack: usize,
    ptid: usize,
    tls: usize,
    ctid: usize,
) -> AlienResult<isize> {
    task_domain.do_clone(flag, stack, ptid, tls, ctid)
}

pub fn sys_wait4(
    task_domain: &Arc<dyn TaskDomain>,
    pid: usize,
    status: usize,
    options: usize,
    rusage: usize,
) -> AlienResult<isize> {
    task_domain.do_wait4(pid as isize, status, options as u32, rusage)
}

pub fn sys_execve(
    task_domain: &Arc<dyn TaskDomain>,
    filename_ptr: usize,
    argv_ptr: usize,
    envp_ptr: usize,
) -> AlienResult<isize> {
    task_domain.do_execve(filename_ptr, argv_ptr, envp_ptr)
}

pub fn sys_yield(scheduler_domain: &Arc<dyn SchedulerDomain>) -> AlienResult<isize> {
    scheduler_domain.yield_now()?;
    Ok(0)
}

pub fn sys_set_tid_address(task_domain: &Arc<dyn TaskDomain>, tidptr: usize) -> AlienResult<isize> {
    task_domain.do_set_tid_address(tidptr)
}

pub fn sys_getuid(_task_domain: &Arc<dyn TaskDomain>) -> AlienResult<isize> {
    Ok(0)
}

pub fn sys_set_pgid(_task_domain: &Arc<dyn TaskDomain>) -> AlienResult<isize> {
    Ok(0)
}

pub fn sys_get_pgid(_task_domain: &Arc<dyn TaskDomain>) -> AlienResult<isize> {
    Ok(0)
}

pub fn sys_set_sid(_task_domain: &Arc<dyn TaskDomain>) -> AlienResult<isize> {
    Ok(0)
}

pub fn sys_get_pid(task_domain: &Arc<dyn TaskDomain>) -> AlienResult<isize> {
    task_domain.current_pid().map(|pid| pid as isize)
}

pub fn sys_get_ppid(task_domain: &Arc<dyn TaskDomain>) -> AlienResult<isize> {
    task_domain.current_ppid().map(|ppid| ppid as isize)
}

pub fn sys_get_euid(_task_domain: &Arc<dyn TaskDomain>) -> AlienResult<isize> {
    Ok(0)
}

pub fn sys_get_gid(_task_domain: &Arc<dyn TaskDomain>) -> AlienResult<isize> {
    Ok(0)
}

pub fn sys_get_egid(_task_domain: &Arc<dyn TaskDomain>) -> AlienResult<isize> {
    Ok(0)
}

pub fn sys_get_tid(task_domain: &Arc<dyn TaskDomain>) -> AlienResult<isize> {
    task_domain.current_tid().map(|tid| tid as isize)
}

pub fn sys_exit(task_domain: &Arc<dyn TaskDomain>, status: usize) -> AlienResult<isize> {
    info!("<sys_exit> status: {}", status);
    task_domain.do_exit(status as isize)
}

pub fn sys_exit_group(task_domain: &Arc<dyn TaskDomain>, status: usize) -> AlienResult<isize> {
    info!("<sys_exit_group> status: {}", status);
    task_domain.do_exit(status as isize)
}
