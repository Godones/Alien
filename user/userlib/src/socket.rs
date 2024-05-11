use alloc::string::String;

use pconst::net::{Domain, ShutdownFlag, SocketAddrIn, SocketType};

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

pub fn bind(socket: usize, address: *const SocketAddrIn, address_len: usize) -> isize {
    sys_bind(socket, address as *const usize, address_len)
}

pub fn listen(socket: usize, backlog: usize) -> isize {
    sys_listen(socket, backlog)
}

pub fn accept(socket: usize, address: *mut SocketAddrIn, address_len: *mut usize) -> isize {
    sys_accept(socket, address as *mut usize, address_len)
}

pub fn connect(socket: usize, address: *const SocketAddrIn, address_len: usize) -> isize {
    sys_connect(socket, address as *const usize, address_len)
}

pub fn getsockname(socket: usize, address: *mut SocketAddrIn, address_len: *mut usize) -> isize {
    sys_getsockname(socket, address as *mut usize, address_len)
}

pub fn getpeername(socket: usize, address: *mut SocketAddrIn, address_len: *mut usize) -> isize {
    sys_getpeername(socket, address as *mut usize, address_len)
}

pub fn sendto(
    socket: usize,
    message: *const u8,
    length: usize,
    flags: usize,
    dest_addr: *const SocketAddrIn,
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
    src_addr: *mut SocketAddrIn,
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
