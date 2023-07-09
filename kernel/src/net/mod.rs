

use socket::Socket;
use addr::{Addr, };

use alloc::sync::Arc;
use syscall_table::syscall_func;
use crate::fs::FileLike;
use crate::task::current_task;

pub mod socket;
pub mod addr;
pub mod port;

#[syscall_func(198)]
pub fn sys_socket(domain: usize, socket_type: usize, protocol:usize) -> isize {
    let process = current_task().unwrap();
    let socket = Socket::new(domain, socket_type, protocol);
    if let Ok(fd) = process.add_file(Arc::new(FileLike::Socket(socket))) {
        fd as isize
    } else {
        ErrorNo::EMFILE as isize
    }
}




#[syscall_func(206)]
pub fn sys_sendto(socket: usize, message: *const u8, length: usize, flags:i32, dest_addr: *const usize, dest_len: usize) -> isize {
    let process = current_task().unwrap();
    let slice = unsafe { core::slice::from_raw_parts(message, length) };

    if let Some(socket) = process.get_socket(socket) {
        // 这里不考虑进程切换
        if let Some(write_len) = socket.send_to(slice, flags, dest_addr) {
            return write_len as isize
        } else {
            return ErrorNo::EINVAL as isize;
        }
    } else {
        return ErrorNo::EBADF as isize;
    }

}

#[syscall_func(207)]
pub fn sys_recvfrom(socket: usize, buffer: *mut u8, length: usize, flags:i32, src_addr: *mut usize, address_len: *mut u32) -> isize {
    let process = current_task().unwrap();
    let slice = unsafe { core::slice::from_raw_parts_mut(buffer, length) };

    if let Some(file) = process.get_socket(socket) {
        // 这里不考虑进程切换
        if let Some(read_len) = file.recvfrom(slice, flags, src_addr, address_len){
            return read_len as isize;
        } else {
            return ErrorNo::EINVAL as isize;
        }
    } else {
        return ErrorNo::EBADF as isize;
    }

}




#[repr(C)]
#[derive(Debug)]
pub enum ErrorNo {
    /// 非法操作
    EPERM = -1,
    /// 找不到文件或目录
    ENOENT = -2,
    /// 找不到对应进程
    ESRCH = -3,
    /// 错误的文件描述符
    EBADF = -9,
    /// 资源暂时不可用。也可因为 futex_wait 时对应用户地址处的值与给定值不符
    EAGAIN = -11,
    /// 无效地址
    EFAULT = -14,
    /// 设备或者资源被占用
    EBUSY = -16,
    /// 文件已存在
    EEXIST = -17,
    /// 不是一个目录(但要求需要是一个目录)
    ENOTDIR = -20,
    /// 是一个目录(但要求不能是)
    EISDIR = -21,
    /// 非法参数
    EINVAL = -22,
    /// fd（文件描述符）已满
    EMFILE = -24,
    /// 对文件进行了无效的 seek
    ESPIPE = -29,
    /// 超过范围。例如用户提供的buffer不够长
    ERANGE = -34,
}