use alloc::vec::Vec;
use core::cmp::min;

use bit_field::BitField;

use pconst::signal::{SignalNumber, SimpleBitSet};
use pconst::LinuxErrno;
use syscall_table::syscall_func;

use crate::config::MAX_FD_NUM;
use crate::fs::file::FilePollExt;
use crate::task::{current_task, do_suspend};
use crate::timer::TimeSpec;

/// 一个系统调用，实现 IO 端口的复用。一般用于用户程序的一段循环体中，
/// 用于周期性检测一组关注的文件描述符集里是否有需要进行处理的IO事件发生。
///
/// 具体的，pselect6 会周期性检测在 `readfds`、`writefds`、`exceptfds`中的文件描述符，
/// 是否符合可读、可写、发生异常。如果有这样的文件描述符，那么就会记录下来，并使得计数器
/// 自增。如果在一次循环后，发现有需要处理的IO事件，那么 pselect6 会直接返回计数器的值(即
/// 事件个数)，如果一直没有需要处理的IO事件，pselect6 也会在 `timeout` 所指明的一段时间后
/// 返回 0，表示在该段时间内没有接收到需要处理的IO事件。pselect6 还可能因为 `SIGKILL` 而被
/// 打断返回。
///
/// 参数有：
/// + `nfds`: 用于指明需要检测的文件描述符中的最大值 + 1，用于作为下面三个 `fds` 中查询
/// 文件描述符是否符合条件的迭代过程的边界条件。目前在 Aien 中，即使传入的 nfds 大于 64，也会被调整为64。
/// + `readfds`: 指向一个64位的位图(u64)，用于记录哪些文件描述符需要被查询可读状态。在执行操作后，该位图被重用为记录哪些文件描述符有事件需要处理。
/// + `writefds`: 指向一个64位的位图(u64)，用于记录哪些文件描述符需要被查询可写状态。在执行操作后，该位图被重用为记录哪些文件描述符有事件需要处理。
/// + `exceptfds`: 指向一个64位的位图(u64)，用于记录哪些文件描述符需要被查询异常状态。在执行操作后，该位图被重用为记录哪些文件描述符有事件需要处理。
/// + `timeout`: 指明超时的时间限制，是一个 [`TimeSpec`] 结构的指针。根据不同取值，不同的效果如下：
///     - 如果该值为空，那么select会一直等待需要处理的IO事件，永远不会超时；
///     - 如果该值不为空，但内部的时间被设为0时，表示即使没有发现需要处理的IO事件，也直接返回。
///     - 否则按照正常的超时时间计算。
/// + `sigmask`: 用于屏蔽某些信号。目前在 Alien 中未使用。
///
/// 有关位图的设计，以 `readfds` 为例：当要检测 fd 为 i 的文件描述符是否已经准备好读时，需要则将位值置为1，否则将该位值置为0。
/// 在执行操作后，该位图被重用为记录哪些文件描述符有事件需要处理，当有事件需要处理时，该位值置为1，否则置为0。`writefds` 和 `exceptfds` 同理。
///
/// 当因为检测到需要处理的IO事件返回时，pselect6 会返回接收到的需要处理的IO事件的总数;
/// 当因为超时而返回时，pselect6 会返回0；
/// 当因为接收到信号而返回时， pselect6 会返回 EINTR；
/// 当其他情况导致的函数执行异常，pselect6 将直接返回错误码。
///
/// Reference: [pselect](https://www.man7.org/linux/man-pages/man2/select.2.html)
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

    let (wait_time, time_spec) = if timeout != 0 {
        let time_spec = task.transfer_raw_ptr(timeout as *mut TimeSpec);
        warn!("pselect6: timeout = {:#x} ---> {:?}", timeout, time_spec);
        (
            Some(time_spec.to_clock() + TimeSpec::now().to_clock()),
            Some(time_spec.clone()),
        )
    } else {
        (Some(usize::MAX), None)
    };
    // assert!(nfds <= 64);
    let nfds = min(nfds, 64);

    // 这里暂时不考虑 sigmask 的问题

    let ori_readfds = if readfds != 0 {
        let readfds = task.transfer_raw_ptr(readfds as *mut u64);
        *readfds
    } else {
        0
    };
    let ori_writefds = if writefds != 0 {
        let writefds = task.transfer_raw_ptr(writefds as *mut u64);
        *writefds
    } else {
        0
    };
    let ori_exceptfds = if exceptfds != 0 {
        let exceptfds = task.transfer_raw_ptr(exceptfds as *mut u64);
        *exceptfds
    } else {
        0
    };

    // at iperf test, if readfds hav one fd is ok, but writefds is empty,
    // it still return 1 and cause recursion error
    do_suspend();
    loop {
        let mut set = 0;
        // 如果设置了监视是否可读的 fd
        if readfds != 0 {
            let readfds = task.transfer_raw_ptr(readfds as *mut u64);
            warn!(
                "[tid:{}]pselect6: readfds = {:#b}",
                task.get_tid(),
                ori_readfds
            );
            for i in 0..nfds {
                if ori_readfds.get_bit(i) {
                    if let Some(fd) = task.get_file(i) {
                        if fd.ready_to_read() {
                            warn!("pselect6: fd {} ready to read", i);
                            readfds.set_bit(i, true);
                            set += 1;
                        } else {
                            readfds.set_bit(i, false);
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
                ori_writefds
            );
            for i in 0..nfds {
                if ori_writefds.get_bit(i) {
                    if let Some(fd) = task.get_file(i) {
                        if fd.ready_to_write() {
                            warn!("pselect6: fd {} ready to write", i);
                            writefds.set_bit(i, true);
                            set += 1;
                        } else {
                            writefds.set_bit(i, false);
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
            warn!(
                "[tid:{}]pselect6: exceptfds = {:#b}",
                task.get_tid(),
                ori_exceptfds
            );
            for i in 0..nfds {
                if ori_exceptfds.get_bit(i) {
                    if let Some(fd) = task.get_file(i) {
                        if fd.in_exceptional_conditions() {
                            warn!("pselect6: fd {} in exceptional conditions", i);
                            exceptfds.set_bit(i, true);
                            set += 1;
                        } else {
                            exceptfds.set_bit(i, false);
                        }
                    } else {
                        return LinuxErrno::EBADF as isize;
                    }
                }
            }
        }

        if set > 0 {
            // let readfds = task.transfer_raw_ptr(readfds as *mut u64);
            // error!("pselect6: readfds = {:#b}", *readfds);
            // 如果找到满足条件的 fd，则返回找到的 fd 数量
            return set;
        }

        if let Some(time_spec) = time_spec {
            if time_spec.tv_sec == 0 && time_spec.tv_nsec == 0 {
                // 不阻塞
                return 0;
            }
        }

        // 否则暂时 block 住
        do_suspend();

        if let Some(wait_time) = wait_time {
            if wait_time <= TimeSpec::now().to_clock() {
                error!(
                    "select timeout, wait_time = {:#x}, now = {:#x}",
                    wait_time,
                    TimeSpec::now().to_clock()
                );
                return 0;
            }
        }

        // interrupt by signal
        let task_inner = task.access_inner();
        let receiver = task_inner.signal_receivers.lock();
        let res = receiver.have_signal_with_number();
        if res.is_some() && res.unwrap() == SignalNumber::SIGKILL as usize {
            return LinuxErrno::EINTR as isize;
        }
    }
}
