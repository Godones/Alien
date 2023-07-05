use lazy_static::lazy_static;

use kernel_sync::Mutex;
pub use pipe::RingBuffer;
pub use signal::*;
use syscall_define::ipc::FutexOp;
use syscall_define::LinuxErrno;
use syscall_table::syscall_func;

use crate::fs::sys_close;
use crate::ipc::futex::{FutexWaiter, FutexWaitManager};
use crate::task::{current_task, TaskState};
use crate::task::schedule::schedule;
use crate::timer::TimeSpec;

mod pipe;
pub mod signal;
pub mod futex;



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
pub fn sys_futex(uaddr: usize, futex_op: u32, val: u32, val2: usize, uaddr2: usize, val3: u32) -> isize {
    let futex_op = FutexOp::try_from(futex_op).unwrap();
    let task = current_task().unwrap();
    let task_inner = task.access_inner();
    warn!("futex: {:?} {:?} {:?} {:?} {:?} {:?}", uaddr, futex_op, val, val2, uaddr2, val3);
    match futex_op {
        FutexOp::FutexWaitPrivate => {
            let uaddr_ref = task_inner.transfer_raw_ptr(uaddr as *const u32);
            if *uaddr_ref != val {
                return LinuxErrno::EAGAIN as isize;
            }
            // we checkout the timeout
            let wait_time = if val2 != 0 {
                let time_spec = task_inner.transfer_raw_ptr(val2 as *const TimeSpec);
                time_spec.to_clock()
            } else {
                // wait forever
                usize::MAX
            };
            // add to wait queue
            let waiter = FutexWaiter::new(task.clone(), wait_time);
            FUTEX_WAITER.lock().add_waiter(uaddr, waiter);
            // switch to other task
            task.update_state(TaskState::Waiting);
            drop(task_inner);
            schedule();
        }
        FutexOp::FutexWakeOpPrivate => {
            let res = FUTEX_WAITER.lock().wake(uaddr, val as usize);
            if res.is_err() {
                return LinuxErrno::EINVAL as isize;
            }
            return res.unwrap() as isize;
        }
        _ => {
            return LinuxErrno::EINVAL as isize;
        }
    }
    0
}
