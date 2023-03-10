use crate::syscall::{sys_list, sys_read, sys_write};

pub fn read(fd: usize, buf: &mut [u8]) -> isize {
    sys_read(fd, buf.as_mut_ptr(), buf.len())
}
pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf.as_ptr(), buf.len())
}

pub fn readdir(fd: usize, buf: &mut [u8]) -> isize {
    sys_read(fd, buf.as_mut_ptr(), buf.len())
}

pub fn list() -> isize {
    sys_list()
}
