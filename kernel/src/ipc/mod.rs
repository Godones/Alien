//! IPC 进程间通信，目前 Alien 支持管道、共享内存、信号以及futex'等进程间的通信机制。
//!
//! [`futex`] 子模块指明了 Alien 中的 futex (快速用户空间互斥体)结构。
//! [`pipe`] 子模块指明了 Alien 中管道结构。
//! [`shm`] 子模块指明了 Alien 中的共享内存结构。
//! [`signal`] 子模块指明了 Alien 中使用的信号机制。

use alloc::sync::Arc;
use core::sync::atomic::{AtomicI32, Ordering};

use constants::{
    ipc::{FutexOp, RobustList},
    AlienResult, LinuxErrno,
};
use ksync::Mutex;
pub use pipe::*;
pub use shm::*;
pub use signal::*;
use spin::Lazy;
use timer::TimeSpec;

use crate::{
    fs::basic::sys_close,
    ipc::futex::{FutexWaitManager, FutexWaiter},
    task::{current_task, schedule::schedule, TaskState},
};

pub mod futex;
mod pipe;
pub mod shm;
pub mod signal;

/// 一个全局变量，用于记录和管理 futex 的等待队列
pub static FUTEX_WAITER: Lazy<Mutex<FutexWaitManager>> =
    Lazy::new(|| Mutex::new(FutexWaitManager::new()));

/// 一对文件描述符。用于创建管道时，返回管道两个端口的文件描述符。
#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct FdPair {
    fd: [u32; 2],
}

/// 一个系统调用，用于创建管道。管道是一种最基本的IPC机制，作用于有血缘关系的进程之间，完成数据传递。
/// 调用 `sys_pipe` 系统函数即可创建一个管道。 `Alien` 中有关管道的设计可见[`Pipe`]。
///
/// `sys_pipe` 按照传入的 `pipe` 解析出对应的 [`FdPair`] 结构在用户内存中的位置，
/// 并将创建成功的管道的读端赋值给 `fd_pair.fd[0]` ，将管道的写端赋值给 `fd_pair.fd[1]` 。
/// 目前的 `flag` 未发挥作用。
///
/// 若创建管道成功，则会返回 0；若发生创建管道错误，或 `pipe == 0` 会导致函数返回 -1。
#[syscall_func(59)]
pub fn sys_pipe(pipe: *mut u32, _flag: u32) -> AlienResult<isize> {
    if pipe.is_null() {
        return Err(LinuxErrno::EINVAL);
    }
    let process = current_task().unwrap();
    let fd_pair = process.transfer_raw_ptr(pipe as *mut FdPair);
    let (read, write) = make_pipe_file()?;
    let read_fd = process.add_file(read).map_err(|_| LinuxErrno::EMFILE)?;
    let write_fd = process.add_file(write).map_err(|_| LinuxErrno::EMFILE)?;
    fd_pair.fd[0] = read_fd as u32;
    fd_pair.fd[1] = write_fd as u32;
    Ok(0)
}

/// 一个系统调用，将进程中一个已经打开的文件复制一份并分配到一个新的文件描述符中。可以用于IO重定向。
/// `old_fd` 指明进程中一个已经打开的文件的文件描述符。
///
/// 如果传入的 `old_fd` 并不对应一个合法的已打开文件，将会返回 -1；
/// 如果创建新的文件描述符失败，那么会返回 `EMFILE`；
/// 否则创建新的文件描述符成功，返回能够访问已打开文件的新文件描述符。(同时文件描述符分配器会保证新文件描述符是当时情况下所能分配的描述符中最小的)
///
/// Reference: https://man7.org/linux/man-pages/man2/dup.2.html
#[syscall_func(23)]
pub fn sys_dup(old_fd: usize) -> AlienResult<isize> {
    let process = current_task().unwrap();
    let file = process.get_file(old_fd).ok_or(LinuxErrno::EBADF)?;
    let new_fd = process
        .add_file(file.clone())
        .map_err(|_| LinuxErrno::EMFILE)?;
    Ok(new_fd as isize)
}

/// 一个系统调用，将进程中一个已经打开的文件复制一份并分配到一个新的文件描述符中。功能上与 `sys_dup` 大致相同。
///
/// `old_fd` 指明进程中一个已经打开的文件的文件描述符，`new_fd` 指明新的文件描述符。
/// 如果 `new_fd` 已经分配给一个文件，那么将自动关闭 `new_fd` 原来对应的那个文件后，再将复制的文件分配到 `new_fd`。
///
/// 如果传入的 `old_fd` 并不对应一个合法的已打开文件或者创建新的文件描述符失败，都将会返回 -1；
/// 如果 `new_fd` 与 `old_fd` 相等， 那么调用将什么也不进行，直接返回 `new_fd`。
/// 否则创建新的文件描述符成功，返回 `new_fd`。
///
/// Reference: https://man7.org/linux/man-pages/man2/dup.2.html
#[syscall_func(24)]
pub fn sys_dup2(old_fd: usize, new_fd: usize, _flag: usize) -> AlienResult<isize> {
    let process = current_task().unwrap();
    let file = process.get_file(old_fd).ok_or(LinuxErrno::EBADF)?;
    let new_file = process.get_file(new_fd);
    if new_file.is_some() {
        let _ = sys_close(new_fd);
    }
    process
        .add_file_with_fd(file.clone(), new_fd)
        .map_err(|_| LinuxErrno::EMFILE)?;
    Ok(new_fd as isize)
}

static FCOUNT: Mutex<usize> = Mutex::new(0);

/// 一个系统调用，对 futex 进行操作。 有关 `futex` 的相关信息请见 [`futex`]。
///
/// 参数：
/// + `uaddr`: 用户态下共享内存的地址，里面存放的是一个对齐的整型计数器，指向一个 futex。
/// + `futex_op`: 指明操作的类型。具体操作类型可见 [`FutexOp`]。目前 Alien 识别的 futex_op 包括：
///     + FutexOp::FutexWaitPrivate | FutexOp::FutexWait: 先比较 uaddr 上计数器的值和 val 是否相等，如果不相等则将直接返回 `EAGAIN`；否则
/// 该进程将等待在 uaddr 上，并根据 val2 的值确定等待的逻辑。若 val2 值为0，则表示进程一直等待；若 val2 是一个正数，则表示进程将在等待 val2 时间后因超时被唤醒
///     + FutexOp::FutexCmpRequeuePiPrivate: 先比较 uaddr 上计数器的值和 val3 是否相等，如果不相等则将直接返回 `EAGAIN`；否则
/// 唤醒至多 val 个在 uaddr 上等待的进程后，将原来等待在 uaddr 上至多 val2 个进程转移到 uaddr2 上等待，最后返回 唤醒的进程数 + 转移的进程数
///     + FutexOp::FutexRequeuePrivate: 唤醒至多 val 个在 uaddr 上等待的进程后，将原来等待在 uaddr 上至多 val2 个进程转移到 uaddr2 上等待
/// 最后返回 唤醒的进程数 + 转移的进程数
///     + FutexOp::FutexWakePrivate | FutexOp::FutexWake: 唤醒至多 val 个在 uaddr 上等待的进程。最后返回 唤醒的进程数。
/// + `val`: 传入的参数1，将根据 futex_op 发挥不同的作用。
/// + `val2`: 传入的参数2，将根据 futex_op 发挥不同的作用。
/// + `uaddr2`: 传入的地址2，将根据 futex_op 发挥不同的作用。
/// + `val3`: 传入的参数3，将根据 futex_op 发挥不同的作用。
///
/// 在此过程中，如果出现异常，会返回异常类型。
///
/// Reference: [futex](https://man7.org/linux/man-pages/man2/futex.2.html)
#[syscall_func(98)]
pub fn futex(
    uaddr: usize,
    futex_op: u32,
    val: u32,
    val2: usize,
    uaddr2: usize,
    val3: u32,
) -> isize {
    *FCOUNT.lock() += 1;
    let futex_op = FutexOp::try_from(futex_op).unwrap();
    let task = current_task().unwrap();
    let task_inner = task.access_inner();
    warn!(
        "futex: {:?} {:?} {:?} {:?} {:?} {:?}",
        uaddr, futex_op, val, val2, uaddr2, val3
    );
    match futex_op {
        FutexOp::FutexWaitPrivate | FutexOp::FutexWait => {
            let uaddr_ref = task_inner.transfer_raw_ptr_mut(uaddr as *mut i32);
            let uaddr_atomic = AtomicI32::from_mut(uaddr_ref);

            if uaddr_atomic.load(Ordering::SeqCst) != val as i32 {
                error!("FutexWait: uaddr_ref != val");
                return LinuxErrno::EAGAIN as isize;
            }
            // we checkout the timeout
            let wait_time = if val2 != 0 {
                let time_spec = task_inner.transfer_raw_ptr(val2 as *const TimeSpec);
                Some(time_spec.to_clock() + TimeSpec::now().to_clock())
            } else {
                // wait forever
                None
            };
            // add to wait queue
            drop(task_inner);
            warn!("Futex wait time: {:?}", wait_time);
            let timeout_flag = Arc::new(Mutex::new(false));
            let waiter = FutexWaiter::new(task.clone(), wait_time, timeout_flag.clone());
            FUTEX_WAITER.lock().add_waiter(uaddr, waiter);
            // switch to other task
            task.update_state(TaskState::Waiting);
            warn!("Because of futex, we switch to other task");
            schedule();
            // checkout the timeout flag
            let timeout_flag = timeout_flag.lock();
            if *timeout_flag {
                return 0;
            }
        }
        FutexOp::FutexCmpRequeuePiPrivate => {
            let uaddr_ref = task_inner.transfer_raw_ptr(uaddr as *const u32);
            if *uaddr_ref != val3 {
                error!("FutexRequeuePrivate: uaddr_ref != val");
                return LinuxErrno::EAGAIN as isize;
            }
            // wake val tasks
            let res = FUTEX_WAITER.lock().wake(uaddr, val as usize);
            if res.is_err() {
                return LinuxErrno::EINVAL as isize;
            }
            // requeue val2 tasks to uaddr2
            let res2 = FUTEX_WAITER.lock().requeue(uaddr2, val2, uaddr);
            if res2.is_err() {
                return LinuxErrno::EINVAL as isize;
            }
            return res2.unwrap() as isize + res.unwrap() as isize;
        }
        FutexOp::FutexRequeuePrivate => {
            // wake val tasks
            let res = FUTEX_WAITER.lock().wake(uaddr, val as usize);
            if res.is_err() {
                return LinuxErrno::EINVAL as isize;
            }
            // requeue val2 tasks to uaddr2
            let res2 = FUTEX_WAITER.lock().requeue(uaddr2, val2, uaddr);
            if res2.is_err() {
                return LinuxErrno::EINVAL as isize;
            }
            return res2.unwrap() as isize + res.unwrap() as isize;
        }
        FutexOp::FutexWakePrivate | FutexOp::FutexWake => {
            let res = FUTEX_WAITER.lock().wake(uaddr, val as usize);
            if res.is_err() {
                return LinuxErrno::EINVAL as isize;
            }
            return res.unwrap() as isize;
        }
        _ => {
            panic!("futex: unimplemented futex_op: {:?}", futex_op);
            // return LinuxErrno::EINVAL as isize;
        }
    }
    0
}

/// 一个系统调用，用于设置当前进程的 robust 锁的列表头。robust 锁主要是解决当一个持有互斥锁的线程退出之后这个锁成为不可用状态的问题。
///
/// 当传入的 `len` 不等于 `HEAD_SIZE` 时，将会返回 `EINVAL`，否则函数将把 `head` 赋值给 tcb 的 robust 的 head 字段，然后返回 0。
#[syscall_func(99)]
pub fn set_robust_list(head: usize, len: usize) -> isize {
    if len != RobustList::HEAD_SIZE {
        return LinuxErrno::EINVAL as isize;
    }
    let task = current_task().unwrap();
    let mut task_inner = task.access_inner();
    task_inner.robust.head = head;
    0
}

/// 一个系统调用，用于获取某进程的 robust 锁的列表头。robust 锁主要是解决当一个持有互斥锁的线程退出之后这个锁成为不可用状态的问题。
///
/// `pid` 指明了要获取相关信息的进程号；`head_ptr` 指明了获取信息后保存的位置；`len_ptr` 指明了获取列表长度信息后保存的位置。
///
/// 当 `pid` 的值为 0 时，将会导致函数 panic；当函数正确执行时，返回 0。
#[syscall_func(100)]
pub fn get_robust_list(pid: usize, head_ptr: usize, len_ptr: usize) -> isize {
    assert_eq!(pid, 0);
    let task = current_task().unwrap();
    let task_inner = task.access_inner();
    let head = task_inner.robust.head;
    let len = RobustList::HEAD_SIZE;
    let head_ref = task_inner.transfer_raw_ptr_mut(head_ptr as *mut usize);
    let len_ref = task_inner.transfer_raw_ptr_mut(len_ptr as *mut usize);
    *head_ref = head;
    *len_ref = len;
    0
}

/// 唤醒所有当前正在等待 futex 但因为超时或者信号而需要被唤醒的进程
pub fn solve_futex_wait() {
    let mut futex_waiter = FUTEX_WAITER.lock();
    futex_waiter.wake_for_timeout();
    futex_waiter.wake_for_signal();
}
