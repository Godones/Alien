//! Pipe

mod pipe;

use crate::task::current_process;
use syscall_table::syscall_func;

use crate::fs::sys_close;
pub use pipe::RingBuffer;

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
    let process = current_process().unwrap();
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
    let process = current_process().unwrap();
    let file = process.get_file(old_fd);
    if file.is_none() {
        return -1;
    }
    let file = file.unwrap();
    let new_fd = process.add_file(file.clone());
    if new_fd.is_err() {
        return -1;
    }
    new_fd.unwrap() as isize
}

#[syscall_func(24)]
pub fn sys_dup2(old_fd: usize, new_fd: usize, _flag: usize) -> isize {
    let process = current_process().unwrap();
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
