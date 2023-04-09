use crate::syscall::*;
pub fn setxattr(path: &str, name: &str, value: &[u8], flag: usize) -> isize {
    sys_setxattr(
        path.as_ptr(),
        name.as_ptr(),
        value.as_ptr(),
        value.len(),
        flag,
    )
}

pub fn lsetxattr(path: &str, name: &str, value: &[u8], flag: usize) -> isize {
    sys_lsetxattr(
        path.as_ptr(),
        name.as_ptr(),
        value.as_ptr(),
        value.len(),
        flag,
    )
}

pub fn fsetxattr(fd: usize, name: &str, value: &[u8], flag: usize) -> isize {
    sys_fsetxattr(fd, name.as_ptr(), value.as_ptr(), value.len(), flag)
}

pub fn getxattr(path: &str, name: &str, value: &mut [u8]) -> isize {
    sys_getxattr(
        path.as_ptr(),
        name.as_ptr(),
        value.as_mut_ptr(),
        value.len(),
    )
}

pub fn lgetxattr(path: &str, name: &str, value: &mut [u8]) -> isize {
    sys_lgetxattr(
        path.as_ptr(),
        name.as_ptr(),
        value.as_mut_ptr(),
        value.len(),
    )
}

pub fn fgetxattr(fd: usize, name: &str, value: &mut [u8]) -> isize {
    sys_fgetxattr(fd, name.as_ptr(), value.as_mut_ptr(), value.len())
}

pub fn listxattr(path: &str, list: &mut [u8]) -> isize {
    sys_listxattr(path.as_ptr(), list.as_mut_ptr(), list.len())
}

pub fn llistxattr(path: &str, list: &mut [u8]) -> isize {
    sys_llistxattr(path.as_ptr(), list.as_mut_ptr(), list.len())
}

pub fn flistxattr(fd: usize, list: &mut [u8]) -> isize {
    sys_flistxattr(fd, list.as_mut_ptr(), list.len())
}

pub fn removexattr(path: &str, name: &str) -> isize {
    sys_removexattr(path.as_ptr(), name.as_ptr())
}

/// Need Test
pub fn lremovexattr(path: &str, name: &str) -> isize {
    sys_lremovexattr(path.as_ptr(), name.as_ptr())
}

/// Need Test
pub fn fremovexattr(fd: usize, name: &str) -> isize {
    sys_fremovexattr(fd, name.as_ptr())
}


pub fn truncate(path: &str, len: usize) -> isize {
    sys_truncate(path.as_ptr(), len)
}

pub fn ftruncate(fd: usize, len: usize) -> isize {
    sys_ftruncate(fd, len)
}