use crate::task::current_task;

/// (待实现)一个系统调用，设置进程调度的参数。目前直接返回0。
#[syscall_func(118)]
pub fn sched_setparam() -> isize {
    0
}

/// (待实现)一个系统调用，获取进程调度的参数。目前直接返回0。
#[syscall_func(121)]
pub fn sched_getparam() -> isize {
    0
}

/// (待实现)一个系统调用，设置进程CPU亲和力(位掩码)，使进程绑定在某一个或几个CPU上运行，避免在CPU之间来回切换，从而提高该进程的实时性能。目前直接返回0。
#[syscall_func(122)]
pub fn sched_setaffinity() -> isize {
    0
}

/// (待完善)一个系统调用，获取某进程对CPU的亲和力(位掩码)。当前进程的cpu亲和力将保存到`mask`所指向的位置。函数执行成功后返回8。
#[syscall_func(123)]
pub fn sched_getaffinity(pid: usize, size: usize, mask: usize) -> isize {
    warn!(
        "sched_getaffinity: pid: {}, size: {}, mask: {}",
        pid, size, mask
    );
    assert_eq!(pid, 0);
    let task = current_task().unwrap();
    let res = task.access_inner().cpu_affinity;
    let mask = task.access_inner().transfer_raw_ptr_mut(mask as *mut usize);
    *mask = res;
    8
}

/// (待实现)一个系统调用，用于获取当前CPU的调度策略。目前直接返回0。
#[syscall_func(120)]
pub fn sched_getscheduler(pid: usize) -> isize {
    assert_eq!(pid, 0);
    // let task = current_task().unwrap();
    0
}

/// (待实现)一个系统调用，用于设置当前CPU的调度策略。目前直接返回0。
#[syscall_func(119)]
pub fn sched_setscheduler(_pid: usize, _policy: usize, _param: usize) -> isize {
    0
}
