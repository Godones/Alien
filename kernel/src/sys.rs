use core::cmp::min;

use syscall_define::sys::{Rusage, Sysinfo, SyslogAction, TimeVal};
use syscall_define::LinuxErrno;
use syscall_table::syscall_func;

use crate::task::{current_task, TASK_MANAGER};
use crate::timer::{get_time_ms, TimeFromFreq};
use crate::MACHINE_INFO;

const LOG_BUF_LEN: usize = 4096;
const LOG: &str = r"
[    0.000000] Linux version 5.10.0-7-riscv64 (debian-kernel@lists.debian.org) (gcc-10 (Debian 10.2.1-6) 10.2.1 20210110, GNU ld (GNU Binutils for Debian) 2.35.2) #1 SMP Debian 5.10.40-1 (2021-05-28)
";

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
    fn ekernel();
}

#[syscall_func(179)]
pub fn sys_info(dst_info: usize) -> isize {
    const LINUX_SYSINFO_LOADS_SCALE: usize = 65536;
    let task = current_task().unwrap();
    // calculate the task number
    // TASKMANAGER
    let task_number = TASK_MANAGER.lock().len();
    let memory_info = MACHINE_INFO.get().as_ref().unwrap().memory.clone();
    let info = Sysinfo {
        uptime: (get_time_ms() / 1000) as usize,
        loads: [
            task_number * LINUX_SYSINFO_LOADS_SCALE / 60,
            task_number * LINUX_SYSINFO_LOADS_SCALE / 300,
            task_number * LINUX_SYSINFO_LOADS_SCALE / 900,
        ],
        totalram: memory_info.end - memory_info.start,
        freeram: memory_info.end - ekernel as usize,
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

#[syscall_func(118)]
pub fn sched_setparam() -> isize {
    0
}

#[syscall_func(121)]
pub fn sched_getparam() -> isize {
    0
}

#[syscall_func(122)]
pub fn sched_setaffinity() -> isize {
    0
}

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

#[syscall_func(120)]
pub fn sched_getscheduler(pid: usize) -> isize {
    assert_eq!(pid, 0);
    // let task = current_task().unwrap();
    0
}

#[syscall_func(119)]
pub fn sched_setscheduler(_pid: usize, _policy: usize, _param: usize) -> isize {
    0
}

#[syscall_func(165)]
pub fn getrusage(who: usize, usage: usize) -> isize {
    warn!("getrusage: who: {}, usage: {}", who, usage);
    if who != 0 {
        panic!("[sys_getrusage] parameter 'who' is not RUSAGE_SELF.");
    }
    let task = current_task().unwrap();
    let static_info = task.access_inner().statistical_data().clone();
    let mut task_usage = Rusage::new();
    task_usage.ru_utime = TimeVal::from_freq(static_info.tms_utime);
    task_usage.ru_stime = TimeVal::from_freq(static_info.tms_stime);
    task.access_inner()
        .copy_to_user(&task_usage, usage as *mut Rusage);
    0
}
