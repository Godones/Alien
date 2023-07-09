use alloc::sync::Arc;

use lazy_static::lazy_static;

use kernel_sync::Mutex;
pub use pipe::RingBuffer;
pub use signal::*;
use syscall_define::ipc::{FutexOp, RobustList};
use syscall_define::LinuxErrno;
use syscall_table::syscall_func;

use crate::fs::sys_close;
use crate::ipc::futex::{FutexWaitManager, FutexWaiter};
use crate::task::schedule::schedule;
use crate::task::{current_task, TaskState};
use crate::timer::TimeSpec;

pub mod futex;
mod pipe;
pub mod signal;

lazy_static! {
    pub static ref FUTEX_WAITER: Mutex<FutexWaitManager> = Mutex::new(FutexWaitManager::new());
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct FdPair {
    fd: [u32; 2],
}

#[syscall_func(59)]
pub fn sys_pipe(pipe: *mut u32, _flag: u32) -> isize {
    if pipe.is_null() {
        return -1;
    }
    let process = current_task().unwrap();
    let fd_pair = process.transfer_raw_ptr(pipe as *mut FdPair);
    let (read, write) = pipe::Pipe::new();
    let read_fd = process.add_file(read);
    if read_fd.is_err() {
        return -1;
    }
    let write_fd = process.add_file(write);
    if write_fd.is_err() {
        return -1;
    }
    fd_pair.fd[0] = read_fd.unwrap() as u32;
    fd_pair.fd[1] = write_fd.unwrap() as u32;
    0
}

/// Reference: https://man7.org/linux/man-pages/man2/dup.2.html
#[syscall_func(23)]
pub fn sys_dup(old_fd: usize) -> isize {
    let process = current_task().unwrap();
    let file = process.get_file(old_fd);
    if file.is_none() {
        return -1;
    }
    let file = file.unwrap();
    let new_fd = process.add_file(file.clone());
    if new_fd.is_err() {
        return LinuxErrno::EMFILE as isize;
    }
    new_fd.unwrap() as isize
}

#[syscall_func(24)]
pub fn sys_dup2(old_fd: usize, new_fd: usize, _flag: usize) -> isize {
    let process = current_task().unwrap();
    let file = process.get_file(old_fd);
    if file.is_none() {
        return -1;
    }
    let file = file.unwrap();
    let new_file = process.get_file(new_fd);
    if new_file.is_some() {
        sys_close(new_fd);
    }
    let result = process.add_file_with_fd(file.clone(), new_fd);
    if result.is_err() {
        return -1;
    }
    new_fd as isize
}

#[syscall_func(98)]
pub fn sys_futex(
    uaddr: usize,
    futex_op: u32,
    val: u32,
    val2: usize,
    uaddr2: usize,
    val3: u32,
) -> isize {
    let futex_op = FutexOp::try_from(futex_op).unwrap();
    let task = current_task().unwrap();
    let task_inner = task.access_inner();
    warn!(
        "futex: {:?} {:?} {:?} {:?} {:?} {:?}",
        uaddr, futex_op, val, val2, uaddr2, val3
    );
    let timeout_flag = Arc::new(Mutex::new(false));
    match futex_op {
        FutexOp::FutexWaitPrivate | FutexOp::FutexWait => {
            let uaddr_ref = task_inner.transfer_raw_ptr(uaddr as *const u32);
            if *uaddr_ref != val {
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
            let waiter = FutexWaiter::new(task.clone(), wait_time, timeout_flag.clone());
            FUTEX_WAITER.lock().add_waiter(uaddr, waiter);
            // switch to other task
            task.update_state(TaskState::Waiting);
            warn!("Because of futex, we switch to other task");
            schedule();
            // checkout the timeout flag
            let timeout_flag = timeout_flag.lock();
            if *timeout_flag {
                return -1;
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
        FutexOp::FutexWakePrivate => {
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

pub fn solve_the_futex_wait_time() {
    let mut futex_waiter = FUTEX_WAITER.lock();
    futex_waiter.wake_for_timeout()
}
