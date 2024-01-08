use crate::task::current_task;
use crate::timer::get_time_ms;
use crate::MACHINE_INFO;
use constants::sys::{Sysinfo, SyslogAction};
use constants::LinuxErrno;
use core::cmp::min;

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
    fn ekernel();
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
