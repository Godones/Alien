use alloc::vec::Vec;
use core::cmp::min;

use bit_field::BitField;

use syscall_define::signal::{SignalNumber, SimpleBitSet};
use syscall_define::LinuxErrno;
use syscall_table::syscall_func;

use crate::config::MAX_FD_NUM;
use crate::fs::file::FilePollExt;
use crate::task::{current_task, do_suspend};
use crate::timer::TimeSpec;

#[syscall_func(72)]
pub fn pselect6(
    nfds: usize,
    readfds: usize,
    writefds: usize,
    exceptfds: usize,
    timeout: usize, // pselect 不会更新 timeout 的值，而 select 会
    sigmask: usize,
) -> isize {
    if nfds >= MAX_FD_NUM {
        return LinuxErrno::EINVAL as isize;
    }
    warn!("pselect6: nfds = {}, readfds = {:#x}, writefds = {:#x}, exceptfds = {:#x}, timeout = {:#x}, sigmask = {:#x}",
        nfds, readfds, writefds, exceptfds, timeout, sigmask);

    // 注意 pselect 不会修改用户空间中的 timeout，所以需要内核自己记录
    let task = current_task().unwrap().clone();

    if sigmask != 0 {
        let mask = task.access_inner().transfer_raw_ptr(sigmask as *mut usize);
        let mask_num: Vec<SignalNumber> = SimpleBitSet(*mask).into();
        error!("pselect6: sigmask = {} ---> {:?}, ", *mask, mask_num);
    }

    let wait_time = if timeout != 0 {
        let time_spec = task.transfer_raw_ptr(timeout as *mut TimeSpec);
        Some(time_spec.to_clock() + TimeSpec::now().to_clock())
    } else {
        // wait forever
        None
    };
    // assert!(nfds <= 64);
    let nfds = min(nfds, 64);

    // 这里暂时不考虑 sigmask 的问题
    loop {
        let mut set = 0;
        // 如果设置了监视是否可读的 fd
        if readfds != 0 {
            let readfds = task.transfer_raw_ptr(readfds as *mut u64);
            warn!(
                "[tid:{}]pselect6: readfds = {:#b}",
                task.get_tid(),
                *readfds
            );
            for i in 0..nfds {
                if readfds.get_bit(i) {
                    if let Some(fd) = task.get_file(i) {
                        if fd.ready_to_read() {
                            warn!("pselect6: fd {} ready to read", i);
                            readfds.set_bit(i, true);
                            set += 1;
                        } else {
                            // readfds.set_bit(i, false);
                        }
                    } else {
                        return LinuxErrno::EBADF as isize;
                    }
                }
            }
        }
        // 如果设置了监视是否可写的 fd
        if writefds != 0 {
            let writefds = task.transfer_raw_ptr(writefds as *mut u64);
            warn!(
                "[tid:{}]pselect6: writefds = {:#b}",
                task.get_tid(),
                *writefds
            );
            for i in 0..nfds {
                if writefds.get_bit(i) {
                    if let Some(fd) = task.get_file(i) {
                        if fd.ready_to_write() {
                            warn!("pselect6: fd {} ready to write", i);
                            writefds.set_bit(i, true);
                            set += 1;
                        } else {
                            // writefds.set_bit(i, false);
                        }
                    } else {
                        return LinuxErrno::EBADF as isize;
                    }
                }
            }
        }
        // 如果设置了监视是否异常的 fd
        if exceptfds != 0 {
            let exceptfds = task.transfer_raw_ptr(exceptfds as *mut u64);
            for i in 0..nfds {
                if exceptfds.get_bit(i) {
                    if let Some(fd) = task.get_file(i) {
                        if fd.in_exceptional_conditions() {
                            warn!("pselect6: fd {} in exceptional conditions", i);
                            exceptfds.set_bit(i, true);
                            set += 1;
                        } else {
                            // exceptfds.set_bit(i, false);
                        }
                    } else {
                        return LinuxErrno::EBADF as isize;
                    }
                }
            }
        }

        if set > 0 {
            // 如果找到满足条件的 fd，则返回找到的 fd 数量
            return set;
        }
        if let Some(wait_time) = wait_time {
            if wait_time <= TimeSpec::now().to_clock() {
                error!("select timeout");
                return 0;
            }
        }

        // 否则暂时 block 住
        do_suspend();
        // interrupt by signal
        let task_inner = task.access_inner();
        let receiver = task_inner.signal_receivers.lock();
        if receiver.have_signal() {
            return LinuxErrno::EINTR.into();
        }
    }
}
