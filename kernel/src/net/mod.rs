use socket::Socket;
// use addr::{Addr, };

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
pub fn sys_sendto(socket: usize, message: *const u8, length: usize, flags:i32, dest_addr: *const usize, _dest_len: usize) -> isize {
    let process = current_task().unwrap();
    let slice = unsafe { core::slice::from_raw_parts(message, length) };

    if let Some(socket) = process.get_socket(socket) {
        if socket.wr_type == SOCKET_WRTYPE::RD_ONLY || socket.wr_type == SOCKET_WRTYPE::CLOSE {
            return ErrorNo::EPERM as isize
        }
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

    if let Some(socket) = process.get_socket(socket) {
        if socket.wr_type == SOCKET_WRTYPE::WR_ONLY || socket.wr_type == SOCKET_WRTYPE::CLOSE {
            return ErrorNo::EPERM as isize
        }
        if let Some(read_len) = socket.recvfrom(slice, flags, src_addr, address_len){
            return read_len as isize;
        } else {
            return ErrorNo::EINVAL as isize;
        }
    } else {
        return ErrorNo::EBADF as isize;
    }

}

#[syscall_func(210)]
pub fn sys_shutdown(socket: usize, how: usize) -> isize {
    let process = current_task().unwrap();
    let sdflag = match how {
        0 => ShutdownFlag::SHUTRD,
        1 => ShutdownFlag::SHUTWR,
        2 => ShutdownFlag::SHUTRDWR,
        _ => return -1,
    };
    if let Some(socket) = process.get_socket(socket) {
        socket.shutdown(sdflag)
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




#[derive(Debug)]
pub enum ADDRFAMILY {
    /// 本地域套接字，用于IPC
    AFUNIX = 1,
    /// 网络域套接字IPV4，用于跨机器之间的通信
    AFINET = 2,
    /// 不指明地址域
    AFUNSPEC = 0,
}

#[derive(Debug, PartialEq)]
pub enum SOCKET_TYPE {
    /// TCP流
    SOCKSTREAM = 1,
    /// UDP数据报
    SOCKDGRAM = 2,
    /// 供一个顺序确定的，可靠的，双向基于连接的套接字
    SOCKSEQPACKET = 5,
}


#[derive(Debug, PartialEq)]
pub enum SOCKET_WRTYPE {
    /// socket的发送和接收信息的功能都被关闭，正处于等待关闭状态
    CLOSE = 0,
    /// socket只能接收消息，发送消息的功能被关闭
    RD_ONLY = 1,
    /// socket只能发送消息，接收消息的功能被关闭
    WR_ONLY = 2,
    /// socket发送和接收信息的功能都正常开启
    RDWR = 3,
}


#[derive(Debug)]
pub enum ShutdownFlag {
    /// 禁用接收
    SHUTRD = 0,
    /// 禁用传输
    SHUTWR = 1,
    /// 同时禁用socket的的传输和接收功能
    SHUTRDWR = 2,
}