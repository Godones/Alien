use bitflags::bitflags;
use crate::syscall::{sys_close, sys_list, sys_open, sys_read, sys_write};

bitflags! {
    pub struct OpenFlags:u32{
        const O_RDONLY = 0;
        const O_WRONLY = 1;
        const O_RDWR = 2;
        const O_CREAT = 0100;
        const O_EXCL = 0200;
        const O_NOCTTY = 0400;
        const O_TRUNC = 01000;
        const O_APPEND = 02000;
        const O_NONBLOCK = 04000;
    }
}
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

pub fn open(name:&str,flag:OpenFlags)->isize{
    sys_open(name.as_ptr(),flag.bits as usize)
}

pub fn close(fd:usize)->isize{
    sys_close(fd)
}