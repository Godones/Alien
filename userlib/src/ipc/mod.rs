use crate::syscall::{sys_dup, sys_dup3, sys_pipe};


pub fn pipe(fd: &mut [u32; 2]) -> isize {
    sys_pipe(fd.as_mut_ptr(), 0)
}


pub fn dup(old_fd: usize) -> isize {
    sys_dup(old_fd)
}

pub fn dup2(old_fd: usize, new_fd: usize) -> isize {
    sys_dup3(old_fd, new_fd, 0)
}

pub fn dup3(old_fd: usize, new_fd: usize, flag: usize) -> isize {
    sys_dup3(old_fd, new_fd, flag)
}