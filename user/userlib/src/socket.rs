use alloc::string::String;

use crate::syscall::{
    sys_accept, sys_bind, sys_connect, sys_getpeername, sys_getsockname, sys_getsockopt,
    sys_listen, sys_recvfrom, sys_sendto, sys_setsockopt, sys_shutdown, sys_socket,
    sys_socket_pair,
};

pub fn socket(domain: Domain, socket_type: SocketType, protocol: usize) -> isize {
    sys_socket(domain as usize, socket_type as usize, protocol)
}

pub fn socket_pair(
    domain: Domain,
    socket_type: SocketType,
    protocol: usize,
    sv: *const usize,
) -> isize {
    sys_socket_pair(domain as usize, socket_type as usize, protocol, sv)
}

pub fn bind(socket: usize, address: *const Sockaddr, address_len: usize) -> isize {
    sys_bind(socket, address as *const usize, address_len)
}

pub fn listen(socket: usize, backlog: usize) -> isize {
    sys_listen(socket, backlog)
}

pub fn accept(socket: usize, address: *mut Sockaddr, address_len: *mut usize) -> isize {
    sys_accept(socket, address as *mut usize, address_len)
}

pub fn connect(socket: usize, address: *const Sockaddr, address_len: usize) -> isize {
    sys_connect(socket, address as *const usize, address_len)
}

pub fn getsockname(socket: usize, address: *mut Sockaddr, address_len: *mut usize) -> isize {
    sys_getsockname(socket, address as *mut usize, address_len)
}

pub fn getpeername(socket: usize, address: *mut Sockaddr, address_len: *mut usize) -> isize {
    sys_getpeername(socket, address as *mut usize, address_len)
}

pub fn sendto(
    socket: usize,
    message: *const u8,
    length: usize,
    flags: usize,
    dest_addr: *const Sockaddr,
    dest_len: usize,
) -> isize {
    sys_sendto(
        socket,
        message,
        length,
        flags,
        dest_addr as *const usize,
        dest_len,
    )
}

pub fn send(socket: usize, message: *const u8, length: usize, flags: usize) -> isize {
    sys_sendto(socket, message, length, flags, 0 as *const usize, 0)
}

pub fn recvfrom(
    socket: usize,
    buffer: *mut u8,
    length: usize,
    flags: usize,
    src_addr: *mut Sockaddr,
    address_len: *mut usize,
) -> isize {
    sys_recvfrom(
        socket,
        buffer,
        length,
        flags,
        src_addr as *mut usize,
        address_len,
    )
}

pub fn recv(socket: usize, buffer: *mut u8, length: usize, flags: usize) -> isize {
    sys_recvfrom(
        socket,
        buffer,
        length,
        flags,
        0 as *mut usize,
        0 as *mut usize,
    )
}

pub fn setsockopt(socket: usize) -> isize {
    sys_setsockopt()
}

pub fn getsockopt(socket: usize) -> isize {
    sys_getsockopt()
}

pub fn shutdown(socket: usize, how: ShutdownFlag) -> isize {
    sys_shutdown(socket, how as usize)
}

#[repr(usize)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[allow(non_camel_case_types)]
/// Generic musl socket domain.
pub enum Domain {
    /// Local communication
    AF_UNIX = 1,
    /// IPv4 Internet protocols
    AF_INET = 2,
}

#[repr(usize)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[allow(non_camel_case_types)]
/// Generic musl socket type.
pub enum SocketType {
    /// Provides sequenced, reliable, two-way, connection-based byte streams.
    /// An out-of-band data transmission mechanism may be supported.
    SOCK_STREAM = 1,
    /// Supports datagrams (connectionless, unreliable messages of a fixed maximum length).
    SOCK_DGRAM = 2,
    /// Provides raw network protocol access.
    SOCK_RAW = 3,
    /// Provides a reliable datagram layer that does not guarantee ordering.
    SOCK_RDM = 4,
    /// Provides a sequenced, reliable, two-way connection-based data
    /// transmission path for datagrams of fixed maximum length;
    /// a consumer is required to read an entire packet with each input system call.
    SOCK_SEQPACKET = 5,
    /// Datagram Congestion Control Protocol socket
    SOCK_DCCP = 6,
    /// Obsolete and should not be used in new programs.
    SOCK_PACKET = 10,
    /// Set O_NONBLOCK flag on the open fd
    SOCK_NONBLOCK = 0x800,
    /// Set FD_CLOEXEC flag on the new fd
    SOCK_CLOEXEC = 0x80000,
}

#[repr(usize)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ShutdownFlag {
    /// 禁用接收
    SHUTRD = 0,
    /// 禁用传输
    SHUTWR = 1,
    /// 同时禁用socket的的传输和接收功能
    SHUTRDWR = 2,
}

// 暂时使用的是ipv4
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Sockaddr {
    pub sa_family: u16,
    pub sa_port: u16,
    pub sa_addr: u32,
    pub zero: [u8; 8],
}

impl Sockaddr {
    pub fn new(sa_family: Domain, sa_addr: u32, sa_port: u16) -> Self {
        let sa_family: u16 = sa_family as u16;
        Sockaddr {
            sa_family,
            sa_port,
            sa_addr,
            zero: [0u8; 8],
        }
    }
}
