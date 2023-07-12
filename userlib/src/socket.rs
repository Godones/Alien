use crate::syscall::{
    sys_socket, sys_sendto, sys_recvfrom, sys_shutdown
};

pub fn socket(domain: usize, socket_type: usize, protocol: usize) -> isize{
    sys_socket(domain, socket_type, protocol)
}

pub fn sendto(
    socket: usize,
    message: *const u8,
    length: usize, 
    flags:i32, 
    dest_addr: *const usize, 
    dest_len: usize
) -> isize {
    sys_sendto(socket, message, length, flags, dest_addr, dest_len)
}

pub fn recvfrom(
    socket: usize, 
    buffer: *mut u8, 
    length: usize, 
    flags:i32, 
    src_addr: *mut usize, 
    address_len: *mut u32
) -> isize{
    sys_recvfrom(socket, buffer, length, flags, src_addr, address_len)
}

pub fn shutdown(socket: usize, how: usize) -> isize {
    sys_shutdown(socket, how)
}


pub struct IpAddr {
    pub family: u16,
    pub port: u16,
    pub addr: u32,
}





/// ADDR_FAMILY 
/// 不指明地址域，
pub const AF_UNSPEC: usize = 0;
/// 本地域套接字，用于IPC
pub const AF_UNIX: usize = 1;
/// 网络域套接字IPV4，用于跨机器之间的通信
pub const AF_INET: usize = 2;

/// TCP流
pub const SOCK_STREAM: usize = 1;
/// UDP数据报
pub const SOCK_DGRAM: usize = 2;
/// 原始套接字
pub const SOCK_RAW: usize = 3;
// 供一个顺序确定的，可靠的，双向基于连接的套接字
pub const SOCK_SEQPACKET: usize = 5;


// SHUTDOWN_FLAG
// 禁用接收
pub const SHUT_RD: usize = 0;
// 禁用传输
pub const SHUT_WR: usize = 1;
// 同时禁用socket的的传输和接收功能
pub const SHUT_RDWR: usize = 2;