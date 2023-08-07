use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;

use syscall_define::net::{Domain, ShutdownFlag, SocketType, SOCKET_TYPE_MASK};
use syscall_define::LinuxErrno;
use syscall_table::syscall_func;

use crate::fs::file::{FileSocketExt, KFile};
use crate::net::addr::{socket_addr_resolution, RawIpV4Addr};
use crate::net::socket::SocketData;
use crate::task::current_task;
use crate::{error_unwrap, option_unwrap};

pub mod addr;
pub mod port;
pub mod socket;

#[syscall_func(198)]
pub fn socket(domain: usize, s_type: usize, protocol: usize) -> isize {
    let domain = Domain::try_from(domain);
    error_unwrap!(domain, LinuxErrno::EAFNOSUPPORT.into());
    let socket_type = SocketType::try_from(s_type & SOCKET_TYPE_MASK as usize);
    error_unwrap!(socket_type, LinuxErrno::EBADF.into());
    let task = current_task().unwrap();
    let socket = SocketData::new(domain, socket_type, protocol);
    warn!("socket domain: {:?}, type: {:?}", domain, socket_type);
    let file = KFile::new(socket);
    if s_type & SocketType::SOCK_NONBLOCK as usize != 0 {
        file.set_nonblock();
        let socket = file.get_socketdata();
        socket.set_socket_nonblock(true);
        warn!("socket with nonblock");
    }
    if s_type & SocketType::SOCK_CLOEXEC as usize != 0 {
        file.set_close_on_exec();
    }
    let fd = task.add_file(file);
    error_unwrap!(fd, LinuxErrno::EMFILE.into());
    fd as isize
}

#[syscall_func(200)]
pub fn bind(socketfd: usize, sockaddr: usize, len: usize) -> isize {
    let socket_fd = common_socket_syscall(socketfd);
    error_unwrap!(socket_fd, socket_fd.err().unwrap());
    let socket_addr = socket_addr_resolution(sockaddr, len);
    error_unwrap!(socket_addr, socket_addr.err().unwrap());
    let socket = socket_fd.get_socketdata();
    let local_addr = socket.local_addr();
    warn!("{:?} bind to {:?}", local_addr, socket_addr);
    match socket.bind(socket_addr) {
        Ok(()) => 0,
        Err(e) => e.into(),
    }
}

#[syscall_func(201)]
pub fn listening(socketfd: usize, backlog: usize) -> isize {
    let socket_fd = common_socket_syscall(socketfd);
    error_unwrap!(socket_fd, socket_fd.err().unwrap());
    let socket = socket_fd.get_socketdata();
    warn!("socket {:?} listening", socket.local_addr());
    match socket.listening(backlog) {
        Ok(_) => 0,
        Err(e) => e.into(),
    }
}

#[syscall_func(202)]
pub fn accept(socketfd: usize, socket_addr: usize, addr_len: usize) -> isize {
    let socket_fd = common_socket_syscall(socketfd);
    error_unwrap!(socket_fd, socket_fd.err().unwrap());
    let socket = socket_fd.get_socketdata();
    match socket.accept() {
        Ok(file) => {
            let file = KFile::new(file);
            // get peer addr
            if socket_addr != 0 {
                let socket = file.get_socketdata();
                let peer_addr = socket.peer_addr().unwrap();
                let raw_ip_addr = RawIpV4Addr::from(peer_addr);
                let task = current_task().unwrap();
                let addr_len_ref = task
                    .access_inner()
                    .transfer_raw_ptr_mut(addr_len as *mut u32);
                *addr_len_ref = core::mem::size_of::<RawIpV4Addr>() as u32;
                task.access_inner()
                    .copy_to_user(&raw_ip_addr, socket_addr as *mut RawIpV4Addr);
            }
            let task = current_task().unwrap();
            let fd = task.add_file(file);
            error_unwrap!(fd, LinuxErrno::EMFILE.into());
            fd as isize
        }
        Err(e) => e.into(),
    }
}

#[syscall_func(203)]
pub fn connect(socketfd: usize, socket_addr: usize, len: usize) -> isize {
    let socket_addr = socket_addr_resolution(socket_addr, len);
    error_unwrap!(socket_addr, socket_addr.err().unwrap());
    let socket_fd = common_socket_syscall(socketfd);
    error_unwrap!(socket_fd, socket_fd.err().unwrap());
    let socket = socket_fd.get_socketdata();
    let local_addr = socket.local_addr();
    warn!("{:?} connect to ip: {:?}", local_addr, socket_addr);
    match socket.connect(socket_addr) {
        Ok(_) => 0,
        Err(e) => e.into(),
    }
}

#[syscall_func(204)]
pub fn getsockname(socketfd: usize, socket_addr: usize, len: usize) -> isize {
    let socket_fd = common_socket_syscall(socketfd);
    error_unwrap!(socket_fd, socket_fd.err().unwrap());
    let socket = socket_fd.get_socketdata();
    let local_addr = socket.local_addr();
    option_unwrap!(local_addr, LinuxErrno::EINVAL.into());
    warn!("getsockname: {:?}", local_addr);
    let raw_ip_addr = RawIpV4Addr::from(local_addr);
    let task = current_task().unwrap();
    task.access_inner()
        .copy_to_user(&raw_ip_addr, socket_addr as *mut RawIpV4Addr);
    let len_ref = task.access_inner().transfer_raw_ptr_mut(len as *mut u32);
    *len_ref = core::mem::size_of::<RawIpV4Addr>() as u32;
    0
}

#[syscall_func(205)]
pub fn get_peer_name(socketfd: usize, sockaddr: usize, len: usize) -> isize {
    let socket_fd = common_socket_syscall(socketfd);
    error_unwrap!(socket_fd, socket_fd.err().unwrap());
    let socket = socket_fd.get_socketdata();
    let socket_addr = socket.peer_addr();
    option_unwrap!(socket_addr, LinuxErrno::EINVAL.into());
    warn!("get_peer_name: {:?}", socket_addr);
    let raw_ip_addr = RawIpV4Addr::from(socket_addr);
    let task = current_task().unwrap();
    task.access_inner()
        .copy_to_user(&raw_ip_addr, sockaddr as *mut RawIpV4Addr);
    let len_ref = task.access_inner().transfer_raw_ptr_mut(len as *mut u32);
    *len_ref = core::mem::size_of::<RawIpV4Addr>() as u32;
    0
}

#[syscall_func(206)]
pub fn sendto(
    socketfd: usize,
    message: *const u8,
    length: usize,
    flags: usize,
    dest_addr: usize,
    dest_len: usize,
) -> isize {
    assert_eq!(flags, 0);
    let socket_fd = common_socket_syscall(socketfd);
    error_unwrap!(socket_fd, socket_fd.err().unwrap());
    let task = current_task().unwrap();
    let message = task.transfer_buffer(message, length);
    // to vec<u8>
    // todo!(don't need)
    let message = message
        .iter()
        .map(|buf| buf.to_vec())
        .flatten()
        .collect::<Vec<u8>>();
    let socket = socket_fd.get_socketdata();
    match socket.socket_type() {
        SocketType::SOCK_STREAM | SocketType::SOCK_SEQPACKET => {
            if dest_addr != 0 {
                return LinuxErrno::EISCONN.into();
            }
        }
        _ => {
            return LinuxErrno::EOPNOTSUPP.into();
        }
    }
    let socket_addr = if dest_addr != 0 {
        let res = socket_addr_resolution(dest_addr, dest_len);
        error_unwrap!(res, res.err().unwrap());
        Some(res)
    } else {
        None
    };
    warn!(
        "sendto: {:?}, local_addr: {:?}, message len: {}",
        socket_addr,
        socket.local_addr(),
        message.len()
    );
    let send = socket.send_to(message.as_slice(), flags, socket_addr);
    error_unwrap!(send, send.err().unwrap().into());
    send as isize
}

#[syscall_func(207)]
pub fn recvfrom(
    socketfd: usize,
    buffer: *mut u8,
    length: usize,
    flags: usize,
    src_addr: usize,
    addr_len: usize,
) -> isize {
    assert_eq!(flags, 0);
    let socket_fd = common_socket_syscall(socketfd);
    error_unwrap!(socket_fd, socket_fd.err().unwrap());
    let socket = socket_fd.get_socketdata();
    warn!(
        "recvfrom: {:?}, local_addr: {:?}",
        socket.peer_addr(),
        socket.local_addr()
    );
    let mut tmp_buffer = vec![0u8; length];
    let recv_info = socket.recvfrom(tmp_buffer.as_mut_slice(), flags);
    error_unwrap!(recv_info, recv_info.err().unwrap().into());
    let task = current_task().unwrap();
    task.access_inner()
        .copy_to_user_buffer(tmp_buffer.as_ptr(), buffer, recv_info.0);
    if src_addr != 0 {
        let raw_ip_addr = RawIpV4Addr::from(recv_info.1);
        task.access_inner()
            .copy_to_user(&raw_ip_addr, src_addr as *mut RawIpV4Addr);
        let addr_len_ref = task
            .access_inner()
            .transfer_raw_ptr_mut(addr_len as *mut u32);
        *addr_len_ref = core::mem::size_of::<RawIpV4Addr>() as u32;
    }
    recv_info.0 as isize
}

#[syscall_func(208)]
pub fn setsockopt() -> isize {
    0
}

#[syscall_func(209)]
pub fn getsockopt() -> isize {
    0
}

#[syscall_func(210)]
pub fn shutdown(socketfd: usize, how: usize) -> isize {
    let flag = ShutdownFlag::try_from(how);
    error_unwrap!(flag, LinuxErrno::EBADF.into());
    let socket_fd = common_socket_syscall(socketfd);
    error_unwrap!(socket_fd, socket_fd.err().unwrap());
    let socket = socket_fd.get_socketdata();
    warn!(
        "shutdown: {:?}, local_addr: {:?}",
        flag,
        socket.local_addr()
    );
    match socket.shutdown(flag) {
        Ok(_) => 0,
        Err(e) => e.into(),
    }
}

#[syscall_func(199)]
pub fn socket_pair(domain: usize, c_type: usize, proto: usize, sv: usize) -> isize {
    let domain = Domain::try_from(domain);
    error_unwrap!(domain, LinuxErrno::EINVAL.into());
    let c_type = SocketType::try_from(c_type);
    error_unwrap!(c_type, LinuxErrno::EINVAL.into());
    warn!(
        "socketpair: {:?}, {:?}, {:?}, {:?}",
        domain, c_type, proto, sv
    );
    panic!("socketpair");
    // LinuxErrno::EAFNOSUPPORT.into()
}

fn common_socket_syscall(sockfd: usize) -> Result<Arc<KFile>, isize> {
    let task = current_task().unwrap();
    let socket_fd = task.get_file(sockfd);
    option_unwrap!(socket_fd, Err(LinuxErrno::EBADF.into()));
    Ok(socket_fd)
}
