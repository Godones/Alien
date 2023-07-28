use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::min;

use rvfs::file::OpenFlags;

use syscall_define::net::{Domain, ShutdownFlag, SocketType, SOCKET_TYPE_MASK};
use syscall_define::LinuxErrno;
use syscall_table::syscall_func;

use crate::fs::file::{FileSocketExt, KFile};
use crate::net::addr::{socket_addr_resolution, IpAddr, IpV4Addr};
use crate::net::port::{check_connect_map, delete_connect_map, mark_connect_map};
use crate::net::socket::SocketData;
use crate::task::{current_task, do_suspend};
use crate::timer::TimeSpec;

pub mod addr;
pub mod port;
pub mod socket;

#[syscall_func(198)]
pub fn socket(domain: usize, s_type: usize, protocol: usize) -> isize {
    let domain = Domain::try_from(domain);
    if domain.is_err() {
        return LinuxErrno::EAFNOSUPPORT.into();
    }
    let socket_type = SocketType::try_from(s_type & SOCKET_TYPE_MASK as usize);
    if socket_type.is_err() {
        return LinuxErrno::EBADF.into();
    }
    let task = current_task().unwrap();
    let socket_type = socket_type.unwrap();
    let socket = SocketData::new(domain.unwrap(), socket_type, protocol);
    warn!("socket domain: {:?}, type: {:?}", domain, socket_type);
    let file = KFile::new(socket);

    if s_type & SocketType::SOCK_NONBLOCK as usize != 0 {
        file.set_nonblock();
    }
    if s_type & SocketType::SOCK_CLOEXEC as usize != 0 {
        file.set_close_on_exec();
    }

    if let Ok(fd) = task.add_file(file) {
        fd as isize
    } else {
        LinuxErrno::EMFILE as isize
    }
}

#[syscall_func(200)]
pub fn bind(sockfd: usize, sockaddr: usize, len: usize) -> isize {
    let socket_fd = common_socket_syscall(sockfd);
    if socket_fd.is_err() {
        return socket_fd.err().unwrap();
    }
    let socket_fd = socket_fd.unwrap();
    let ip = socket_addr_resolution(sockaddr as *const u16, len);
    if ip.is_err() {
        return ip.err().unwrap();
    }
    let ip = ip.unwrap();
    warn!("bind: {:?}", ip);
    let socket = socket_fd.get_socketdata_mut();
    match socket.bind(ip) {
        Ok(()) => 0,
        Err(e) => e.into(),
    }
}

#[syscall_func(201)]
pub fn listening(sockfd: usize, backlog: usize) -> isize {
    let socket_fd = common_socket_syscall(sockfd);
    if socket_fd.is_err() {
        return socket_fd.err().unwrap();
    }
    let socket_fd = socket_fd.unwrap();
    let socket = socket_fd.get_socketdata_mut();
    socket.listening(backlog);
    0
}

#[syscall_func(202)]
pub fn accept(sockfd: usize, sockaddr: usize, addr_len: usize) -> isize {
    let socket_fd = common_socket_syscall(sockfd);
    if socket_fd.is_err() {
        return socket_fd.err().unwrap();
    }
    let socket_fd = socket_fd.unwrap();
    let socket = socket_fd.get_socketdata_mut();
    loop {
        let mut buf = [0u8; 16]; //first connect query
        let accept = socket.accept(&mut buf);
        if accept.is_none() {
            return LinuxErrno::EINVAL.into();
        }
        let (r, src_ip) = accept.unwrap();
        if r == 0 {
            warn!("accept socket: {:?}, no connect data", sockfd);
            // no data, we need wait according to
            let flag = socket_fd.get_file().access_inner().flags;
            if flag.contains(OpenFlags::O_NONBLOCK) {
                return LinuxErrno::EAGAIN.into();
            }
            do_suspend();
        } else {
            warn!("There is {:?} invoke connect", src_ip);
            let task = current_task().unwrap();
            assert_eq!(r, core::mem::size_of::<IpV4Addr>());
            // set the remote ip
            socket.peer_addr = src_ip.clone();
            let be_src_ip = src_ip.to_be_ipv4().unwrap();
            task.access_inner()
                .copy_to_user(&be_src_ip, sockaddr as *mut IpV4Addr);
            let addr_len = task
                .access_inner()
                .transfer_raw_ptr_mut(addr_len as *mut u32);
            *addr_len = core::mem::size_of::<IpV4Addr>() as u32;

            // mark the connect true
            mark_connect_map(socket.local_addr.clone(), src_ip);

            // make new fd
            let mut new_socket = socket.clone();
            new_socket.listening = false;
            let new_file = SocketData::new_with_data(new_socket);
            let new_fd = task.add_file(KFile::new(new_file));
            if new_fd.is_err() {
                return LinuxErrno::EMFILE.into();
            }
            return new_fd.unwrap() as isize;
        }
    }
}

#[syscall_func(203)]
pub fn connect(sockfd: usize, sockaddr: usize, len: usize) -> isize {
    let ip = socket_addr_resolution(sockaddr as *const u16, len);
    if ip.is_err() {
        return ip.err().unwrap();
    }
    let ip = ip.unwrap();
    warn!("connect socket: {:?}, ip: {:?}", sockfd, ip);
    if ip.port().is_some() && ip.port().unwrap() == 65535 {
        return 0;
    }
    let socket_fd = common_socket_syscall(sockfd);
    if socket_fd.is_err() {
        return socket_fd.err().unwrap();
    }
    let socket_fd = socket_fd.unwrap();
    let socket = socket_fd.get_socketdata_mut();
    match socket.connect(ip.clone()) {
        Ok(_) => {
            let open_flag = socket_fd.get_file().access_inner().flags;
            if open_flag.contains(OpenFlags::O_NONBLOCK) {
                LinuxErrno::EINPROGRESS.into()
            } else {
                let res = connect_check(socket.peer_addr.clone(), socket.local_addr.clone());
                if res.is_ok() {
                    warn!(
                        "{:?} connect {:?} success",
                        socket.local_addr, socket.peer_addr
                    );
                    0
                } else {
                    res.err().unwrap().into()
                }
            }
        }
        Err(e) => e.into(),
    }
}

fn connect_check(server_ip: IpAddr, client_ip: IpAddr) -> Result<(), LinuxErrno> {
    warn!("connect_check: {:?}, {:?}", server_ip, client_ip);
    let wait_time = TimeSpec::now().to_clock() + TimeSpec::new(5, 0).to_clock();
    loop {
        let is_connected = check_connect_map(server_ip.clone(), client_ip.clone());
        if is_connected {
            delete_connect_map(server_ip.clone(), client_ip.clone());
            return Ok(());
        }
        // wait for 5s
        if TimeSpec::now().to_clock() > wait_time {
            return Err(LinuxErrno::EINPROGRESS);
        }
        do_suspend();
        // check signal
        // interrupt by signal
        let task = current_task().unwrap();
        let task_inner = task.access_inner();
        let receiver = task_inner.signal_receivers.lock();
        if receiver.have_signal() {
            return Err(LinuxErrno::EINTR);
        }
    }
}

#[syscall_func(204)]
pub fn getsockname(sockfd: usize, sockaddr: usize, _len: usize) -> isize {
    let socket_fd = common_socket_syscall(sockfd);
    if socket_fd.is_err() {
        return socket_fd.err().unwrap();
    }
    let socket_fd = socket_fd.unwrap();
    let socket = socket_fd.get_socketdata_mut();
    let socket_addr = socket.to_be_ipv4_addr();
    warn!("getsockname: {:?}", socket_addr);
    let task = current_task().unwrap();
    task.access_inner()
        .copy_to_user(&socket_addr, sockaddr as *mut IpV4Addr);
    0
}

#[syscall_func(206)]
pub fn sendto(
    sockfd: usize,
    message: *const u8,
    length: usize,
    flags: usize,
    dest_addr: usize,
    dest_len: usize,
) -> isize {
    assert_eq!(flags, 0);
    let socket_fd = common_socket_syscall(sockfd);
    if socket_fd.is_err() {
        return socket_fd.err().unwrap();
    }
    let socket_fd = socket_fd.unwrap();
    let task = current_task().unwrap();
    let message = task.transfer_buffer(message, length);
    // to vec<u8>
    let message = message
        .iter()
        .map(|buf| buf.to_vec())
        .flatten()
        .collect::<Vec<u8>>();
    let socket = socket_fd.get_socketdata_mut();

    let socket_type = socket.socket_type();
    let dest_addr = match socket_type {
        SocketType::SOCK_STREAM | SocketType::SOCK_SEQPACKET => {
            if dest_addr != 0 {
                return LinuxErrno::EISCONN.into();
            }
            None
        }
        _ => {
            let dest_addr = socket_addr_resolution(dest_addr as *const u16, dest_len);
            if dest_addr.is_err() {
                return dest_addr.err().unwrap();
            }
            Some(dest_addr.unwrap())
        }
    };
    warn!(
        "{:?} sendto: dest: {:?}, message: {:?}",
        socket.local_addr,
        dest_addr,
        message.len()
    );
    if let Some(w) = socket.send_to(message.as_slice(), flags, dest_addr) {
        w as isize
    } else {
        LinuxErrno::EINVAL.into()
    }
}

#[syscall_func(207)]
pub fn recvfrom(
    sockfd: usize,
    buffer: *mut u8,
    length: usize,
    flags: usize,
    src_addr: usize,
    addr_len: usize,
) -> isize {
    warn!(
        "recvfrom: {:?}, {:p}, {:?}, {:?}, {:#x}, {:?}",
        sockfd, buffer, length, flags, src_addr, addr_len
    );
    assert_eq!(flags, 0);
    let socket_fd = common_socket_syscall(sockfd);
    if socket_fd.is_err() {
        return socket_fd.err().unwrap();
    }
    let socket_fd = socket_fd.unwrap();
    let socket = socket_fd.get_socketdata_mut();
    let mut tmp_buffer = vec![0u8; length];
    loop {
        let task = current_task().unwrap();
        let res = socket.recvfrom(tmp_buffer.as_mut_slice(), flags);
        if res.is_some() {
            let (r, src_ip) = res.unwrap();
            let min_r = min(r, length);
            warn!("recvfrom r: {:?}, src_ip: {:?}", r, src_ip);
            task.access_inner()
                .copy_to_user_buffer(tmp_buffer.as_ptr(), buffer, min_r);
            if src_addr != 0 {
                let be_src_ip = src_ip.to_be_ipv4().unwrap();
                task.access_inner()
                    .copy_to_user(&be_src_ip, src_addr as *mut IpV4Addr);
            }
            if addr_len != 0 {
                let addr_len = task
                    .access_inner()
                    .transfer_raw_ptr_mut(addr_len as *mut u32);
                *addr_len = core::mem::size_of::<IpV4Addr>() as u32;
            }
            return min_r as isize;
        } else {
            let flag = socket_fd.get_file().access_inner().flags;
            if flag.contains(OpenFlags::O_NONBLOCK) {
                return LinuxErrno::EAGAIN.into();
            }
            warn!(
                "[tid:{}] {:?} recvfrom: suspend",
                task.get_tid(),
                socket.local_addr
            );
            do_suspend();
            // todo! handler signal
        };
    }
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
pub fn shutdown(sockfd: usize, how: usize) -> isize {
    let sdflag = ShutdownFlag::try_from(how);
    if sdflag.is_err() {
        return LinuxErrno::EBADF.into();
    }
    let sdflag = sdflag.unwrap();
    let socket_fd = common_socket_syscall(sockfd);
    if socket_fd.is_err() {
        return socket_fd.err().unwrap();
    }
    let socket_fd = socket_fd.unwrap();
    let socket = socket_fd.get_socketdata_mut();
    socket.shutdown(sdflag);
    0
}

#[syscall_func(199)]
pub fn socket_pair(domain: usize, c_type: usize, proto: usize, sv: usize) -> isize {
    let domain = Domain::try_from(domain);
    if domain.is_err() {
        return LinuxErrno::EAFNOSUPPORT.into();
    }
    let domain = domain.unwrap();
    let c_type = SocketType::try_from(c_type);
    if c_type.is_err() {
        return LinuxErrno::EBADF.into();
    }
    let c_type = c_type.unwrap();
    warn!(
        "socketpair: {:?}, {:?}, {:?}, {:?}",
        domain, c_type, proto, sv
    );
    LinuxErrno::EAFNOSUPPORT.into()
}

fn common_socket_syscall(sockfd: usize) -> Result<Arc<KFile>, isize> {
    let task = current_task().unwrap();
    let socket_fd = task.get_file(sockfd);
    if socket_fd.is_none() {
        return Err(LinuxErrno::EBADF.into());
    }
    let socket_fd = socket_fd.unwrap();
    Ok(socket_fd)
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
