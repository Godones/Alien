use crate::error_unwrap;
use crate::task::current_task;
use crate::timer::TimeFromFreq;
use syscall_define::sys::{Rusage, RusageFlag, TimeVal};
use syscall_define::LinuxErrno;
/// (待完善)一个系统调用，用于获取对系统资源的使用量信息。获取的信息将保存到`usage`所指向的[`Rusage`]结构中。
///
/// 可以通过`who`修改获取信息的对象，包括:
/// + `RUSAGE_SELF`: 返回调用该函数进程的资源用量统计，会返回该进程下所有线程的资源用量之和;
/// + `RUSAGE_CHILDREN`: 返回调用该函数进程所有已终止且被回收子进程的资源用量统计.
/// + `RUSAGE_THREAD`: 返回调用该函数线程的资源用量统计。
///
/// 在Alien中，目前仅支持`RUSAGE_SELF`。且返回的信息目前仅有[`Rusage`]下的`ru_utime`和`ru_stime`字段。
///
/// 正确执行后返回0。
#[syscall_func(165)]
pub fn getrusage(who: isize, usage: usize) -> isize {
    let who = RusageFlag::try_from(who);
    error_unwrap!(who, LinuxErrno::EINVAL as isize);
    warn!("getrusage: who: {:?}, usage: {}", who, usage);
    let task = current_task().unwrap();
    let static_info = task.access_inner().statistical_data().clone();
    let mut task_usage = Rusage::new();
    task_usage.ru_utime = TimeVal::from_freq(static_info.tms_utime);
    task_usage.ru_stime = TimeVal::from_freq(static_info.tms_stime);
    task.access_inner()
        .copy_to_user(&task_usage, usage as *mut Rusage);
    0
}
