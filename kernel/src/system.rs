//! uname系统调用实现

use alloc::vec;
use core::cmp::min;

use constants::{
    sys::{Rusage, RusageFlag, Sysinfo, SyslogAction},
    time::TimeVal,
    AlienResult, LinuxErrno,
};
use syscall_table::syscall_func;
use timer::{get_time_ms, TimeFromFreq};

use crate::task::current_task;

/// 记录系统信息的结构，包括操作系统名、在网络中的用户名、操作系统release和version版本、硬件类型、域名等信息。
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Utsname {
    /// 操作系统名
    sysname: [u8; 65],
    /// Name within communications network to which the node is attached
    nodename: [u8; 65],
    /// 系统发行版
    release: [u8; 65],
    /// 系统版本
    version: [u8; 65],
    /// 硬件类型
    machine: [u8; 65],
    /// 域名
    domainname: [u8; 65],
}

/// 返回系统信息，信息保存在[`Utsname`]结构中。
fn system_info() -> Utsname {
    const SYSNAME: &str = "Linux";
    const NODENAME: &str = "Alien";
    const RELEASE: &str = "5.1";
    const VERSION: &str = "5.1";
    const MACHINE: &str = "riscv64";
    const DOMAINNAME: &str = "RustOS";
    let mut name = Utsname {
        sysname: [0; 65],
        nodename: [0; 65],
        release: [0; 65],
        version: [0; 65],
        machine: [0; 65],
        domainname: [0; 65],
    };
    name.sysname[..SYSNAME.len()].copy_from_slice(SYSNAME.as_bytes());
    name.nodename[..NODENAME.len()].copy_from_slice(NODENAME.as_bytes());
    name.release[..RELEASE.len()].copy_from_slice(RELEASE.as_bytes());
    name.version[..VERSION.len()].copy_from_slice(VERSION.as_bytes());
    name.machine[..MACHINE.len()].copy_from_slice(MACHINE.as_bytes());
    name.domainname[..DOMAINNAME.len()].copy_from_slice(DOMAINNAME.as_bytes());
    name
}

/// 一个系统调用，返回系统信息。信息包括操作系统名、在网络中的用户名、操作系统release和version版本、硬件类型、域名等信息，详情可见[`Utsname`]。
///
/// 函数成功执行后返回0。
#[syscall_func(160)]
pub fn uname(utsname: *const u8) -> isize {
    let task = current_task().unwrap();
    task.access_inner()
        .copy_to_user(&system_info(), utsname as *mut Utsname);
    0
}

const LOG_BUF_LEN: usize = 4096;
const LOG: &str = r"
[    0.000000] Linux version 5.10.0-7-riscv64 (debian-kernel@lists.debian.org) (gcc-10 (Debian 10.2.1-6) 10.2.1 20210110, GNU ld (GNU Binutils for Debian) 2.35.2) #1 SMP Debian 5.10.40-1 (2021-05-28)
";

/// (待完善)一个系统调用函数，用于对内核消息环状缓冲区进行操作。
///
/// + `log_type`: 指明操作的类型，具体值可见[`SyslogAction`]；
/// + `buf`: 指明读取消息时，消息要保存到的位置；
/// + `len`: 指明具体操作时，对于消息读取的长度限制。真正的读取消息的长度将取决于就传入的`len`和`LOG_BUF_LEN`的最小值。
///
/// 当`log_type`为`READ`、`ReadAll`、`ReadClear`三种flag，正确执行后返回读取消息的长度；
/// 当`log_type`为`Unknown`时，会返回`EINVAL`；当`log_type`为`OPEN`或`CLOSE`时，函数不进行任何操作后返回0。
/// 目前Alien仅支持上述`log_type`值，其余值都将不进行任何操作后返回0。
///
/// Reference: [syslog](https://man7.org/linux/man-pages/man2/syslog.2.html)
#[syscall_func(116)]
pub fn syslog(log_type: u32, buf: usize, len: usize) -> isize {
    let log_type = SyslogAction::try_from(log_type);
    if log_type.is_err() {
        return LinuxErrno::EINVAL as isize;
    }
    match log_type.unwrap() {
        SyslogAction::OPEN | SyslogAction::CLOSE => 0,
        SyslogAction::READ | SyslogAction::ReadAll | SyslogAction::ReadClear => {
            let min_len = min(len, LOG_BUF_LEN);
            let task = current_task().unwrap();
            // the buf may be not valid, so we need to check it -- > sbrk heap
            let mut buf = task.transfer_buffer(buf as *mut u8, min_len);
            let log = LOG.as_bytes();
            let mut offset = 0;
            buf.iter_mut().for_each(|buf| {
                let copy_len = min(log.len() - offset, buf.len());
                buf[..copy_len].copy_from_slice(&log[offset..offset + copy_len]);
                offset += copy_len;
            });
            offset as isize
        }
        SyslogAction::Unknown => LinuxErrno::EINVAL as isize,
        _ => 0,
    }
}

extern "C" {
    fn sheap();
}

/// 一个系统调用函数，用于获取系统相关信息。信息包括系统的自启动经过的时间、对于内存的使用情况、共享存储区的大小、
/// 缓冲区与交换区的大小、当前进程数目等，具体可见[`Sysinfo`]。获取到的信息将保存到`dst_info`所指向的[`Sysinfo`]结构处。
///
/// 目前功能还有待完善。正确执行后返回0。
#[syscall_func(179)]
pub fn sys_info(dst_info: usize) -> isize {
    const LINUX_SYSINFO_LOADS_SCALE: usize = 65536;
    let task = current_task().unwrap();
    // calculate the task number
    let task_number = 10; // fake task number
    let machine_info = platform::platform_machine_info();
    let memory_info = machine_info.memory.clone();
    let info = Sysinfo {
        uptime: (get_time_ms() / 1000) as usize,
        loads: [
            task_number * LINUX_SYSINFO_LOADS_SCALE / 60,
            task_number * LINUX_SYSINFO_LOADS_SCALE / 300,
            task_number * LINUX_SYSINFO_LOADS_SCALE / 900,
        ],
        totalram: memory_info.end - memory_info.start,
        freeram: memory_info.end - sheap as usize,
        sharedram: 0,
        bufferram: 0,
        totalswap: 0,
        freeswap: 0,
        procs: task_number as u16,
        totalhigh: 0,
        freehigh: 0,
        mem_unit: 1,
    };
    task.access_inner()
        .copy_to_user(&info, dst_info as *mut Sysinfo);
    0
}

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
pub fn getrusage(who: isize, usage: usize) -> AlienResult<isize> {
    let who = RusageFlag::try_from(who).map_err(|_| LinuxErrno::EINVAL)?;
    info!("getrusage: who: {:?}, usage: {}", who, usage);
    let task = current_task().unwrap();
    let static_info = task.access_inner().statistical_data().clone();
    let mut task_usage = Rusage::default();
    task_usage.ru_utime = TimeVal::from_freq(static_info.tms_utime);
    task_usage.ru_stime = TimeVal::from_freq(static_info.tms_stime);
    task.access_inner()
        .copy_to_user(&task_usage, usage as *mut Rusage);
    Ok(0)
}

#[syscall_func(278)]
pub fn get_random(buf: *mut u8, len: usize, flags: u32) -> AlienResult<isize> {
    info!(
        "get_random: buf: {:x?}, len: {}, flags: {}",
        buf, len, flags
    );
    let task = current_task().unwrap();
    let mut rand_buf = vec![0; len];
    for v in rand_buf.iter_mut() {
        let time_ms = get_time_ms();
        *v = (time_ms & 0xff) as u8;
    }
    task.access_inner()
        .copy_to_user_buffer(rand_buf.as_ptr(), buf, len);
    Ok(len as isize)
}

/// 一个系统调用，通过调用 SBI_SHUTDOWN 来关闭操作系统（直接退出 QEMU）
#[syscall_func(2003)]
pub fn system_shutdown() -> AlienResult<isize> {
    println!("shutdown...");
    platform::system_shutdown()
}
