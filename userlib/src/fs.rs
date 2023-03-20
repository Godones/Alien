use crate::syscall::{
    sys_chdir, sys_close, sys_get_cwd, sys_list, sys_mkdir, sys_open, sys_read, sys_write,
};
use alloc::string::String;
use bitflags::bitflags;

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

pub fn open(name: &str, flag: OpenFlags) -> isize {
    sys_open(name.as_ptr(), flag.bits as usize)
}

pub fn close(fd: usize) -> isize {
    sys_close(fd)
}

pub fn get_cwd(buf: &mut [u8]) -> Result<&str, IoError> {
    let len = sys_get_cwd(buf.as_mut_ptr(), buf.len());
    if len == -1 {
        return Err(IoError::BufferTooSmall);
    } else {
        let s = core::str::from_utf8(&buf[..len as usize]).unwrap();
        Ok(s)
    }
}

pub fn chdir(path: &str) -> isize {
    sys_chdir(path.as_ptr())
}
pub fn mkdir(path: &str) -> isize {
    sys_mkdir(path.as_ptr())
}
#[derive(Debug)]
pub enum IoError {
    BufferTooSmall,
    FileNotFound,
    FileAlreadyExist,
}
