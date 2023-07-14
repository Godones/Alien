use syscall_define::socket::{Domain, ShutdownFlag, SocketType};
use syscall_define::LinuxErrno;
use syscall_table::syscall_func;

use crate::fs::file::KFile;
use crate::net::socket::SocketData;
use crate::task::current_task;

pub mod addr;
pub mod port;
pub mod socket;

#[syscall_func(198)]
pub fn sys_socket(domain: usize, socket_type: usize, protocol: usize) -> isize {
    let domain = Domain::try_from(domain);
    if domain.is_err() {
        return LinuxErrno::EAFNOSUPPORT.into();
    }
    let socket_type = SocketType::try_from(socket_type);
    if socket_type.is_err() {
        return LinuxErrno::EBADF.into();
    }
    let process = current_task().unwrap();
    let socket = SocketData::new(domain.unwrap(), socket_type.unwrap(), protocol);
    if let Ok(fd) = process.add_file(KFile::new(socket)) {
        fd as isize
    } else {
        LinuxErrno::EMFILE as isize
    }
}

#[syscall_func(206)]
pub fn sys_sendto(
    socket: usize,
    message: *const u8,
    length: usize,
    flags: i32,
    dest_addr: *const usize,
    _dest_len: usize,
) -> isize {
    let process = current_task().unwrap();
    let slice = unsafe { core::slice::from_raw_parts(message, length) };

    // if let Some(socket) = process.get_socket(socket) {
    //     if socket.wr_type == SocketWrtype::RdOnly || socket.wr_type == SocketWrtype::CLOSE {
    //         return LinuxErrno::EPERM as isize;
    //     }
    //     if let Some(write_len) = socket.send_to(slice, flags, dest_addr) {
    //         return write_len as isize;
    //     } else {
    //         return LinuxErrno::EINVAL as isize;
    //     }
    // } else {
    //     return LinuxErrno::EBADF as isize;
    // }
    0
}

#[syscall_func(207)]
pub fn sys_recvfrom(
    socket: usize,
    buffer: *mut u8,
    length: usize,
    flags: i32,
    src_addr: *mut usize,
    address_len: *mut u32,
) -> isize {
    let process = current_task().unwrap();
    let slice = unsafe { core::slice::from_raw_parts_mut(buffer, length) };
    //
    // if let Some(socket) = process.get_socket(socket) {
    //     if socket.wr_type == SocketWrtype::WrOnly || socket.wr_type == SocketWrtype::CLOSE {
    //         return LinuxErrno::EPERM as isize;
    //     }
    //     if let Some(read_len) = socket.recvfrom(slice, flags, src_addr, address_len) {
    //         return read_len as isize;
    //     } else {
    //         return LinuxErrno::EINVAL as isize;
    //     }
    // } else {
    //     return LinuxErrno::EBADF as isize;
    // }
    0
}

#[syscall_func(210)]
pub fn sys_shutdown(socket: usize, how: usize) -> isize {
    let process = current_task().unwrap();
    let sdflag = ShutdownFlag::try_from(how);
    if sdflag.is_err() {
        return LinuxErrno::EBADF.into();
    }
    // if let Some(socket) = process.get_socket(socket) {
    //     socket.shutdown(sdflag)
    // } else {
    //     return LinuxErrno::EBADF as isize;
    // }
    0
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
pub enum SocketWrtype {
    /// socket的发送和接收信息的功能都被关闭，正处于等待关闭状态
    CLOSE = 0,
    /// socket只能接收消息，发送消息的功能被关闭
    RdOnly = 1,
    /// socket只能发送消息，接收消息的功能被关闭
    WrOnly = 2,
    /// socket发送和接收信息的功能都正常开启
    RDWR = 3,
}
