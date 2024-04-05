use alloc::sync::Arc;

use constants::AlienResult;
use interface::TaskDomain;

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

pub fn sys_yield(task_domain: &Arc<dyn TaskDomain>) -> AlienResult<isize> {
    task_domain.do_yield()
}
