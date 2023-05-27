use crate::syscall::{sys_dup, sys_dup3, sys_mmap, sys_munmap, sys_pipe};
use bitflags::bitflags;

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

bitflags! {
    pub struct ProtFlags: u32 {
        const PROT_NONE = 0x0;
        const PROT_READ = 0x1;
        const PROT_WRITE = 0x2;
        const PROT_EXEC = 0x4;
    }
}

bitflags! {
    pub struct MapFlags: u32 {
        const MAP_SHARED = 0x01;
        const MAP_PRIVATE = 0x02;
        const MAP_FIXED = 0x10;
        const MAP_ANONYMOUS = 0x20;
    }
}
pub fn mmap(
    start: usize,
    len: usize,
    prot: ProtFlags,
    flag: MapFlags,
    fd: usize,
    offset: usize,
) -> isize {
    sys_mmap(
        start,
        len,
        prot.bits() as usize,
        flag.bits() as usize,
        fd,
        offset,
    )
}

pub fn munmap(start: usize, len: usize) -> isize {
    sys_munmap(start, len)
}
