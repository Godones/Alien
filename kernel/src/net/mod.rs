//! Alien 内核部分的的网络模块，向下调用 `simple_net` 模块实现 tcp 和 udp 套接字的系统调用。
//! 
//! [`addr`] 子模块指明了在 Alien 内核中使用的 socket 套接字地址结构。
//! [`port`] 子模块现为将网络异常类型 [`NetError`] 转为 系统异常类型 [`LinuxErrno`]的模块。
//! [`socket`] 子模块指明了Alien 内核中使用的套接字。
//! [`unix`] 子模块指明了有关 Unix 协议族下的套接字结构。(目前有关的功能有待支持)
//! 
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;

use syscall_define::net::{
    Domain, ShutdownFlag, SocketLevel, SocketOption, SocketType, TcpSocketOption, SOCKET_TYPE_MASK,
};
use syscall_define::LinuxErrno;
use syscall_table::syscall_func;

use crate::fs::file::{FileSocketExt, KFile};
use crate::net::addr::{socket_addr_resolution, RawIpV4Addr};
use crate::net::socket::SocketData;
use crate::task::{current_task, do_suspend};
use crate::{error_unwrap, option_unwrap};

pub mod addr;
pub mod port;
pub mod socket;
mod unix;

/// 一个系统调用，用于创建一个未绑定的socket套接字。
///
/// + `domain`: 指明套接字被创建的协议簇(包括文件路径协议簇和网络地址协议簇，具体可见[`Domain`]);
/// + `s_type`: 指明被创建的socket的类型，具体可见[`SocketType`];
/// + `protocol`: 指明该socket应用于某一个特定的协议上。当确定了套接字使用的协议簇和类型，该参数可以取为0。
///
/// 如果创建套接字成功则返回一个能在之后使用的文件描述符，否则返回错误信息。
#[syscall_func(198)]
pub fn socket(domain: usize, s_type: usize, protocol: usize) -> isize {
    let domain = Domain::try_from(domain);
    error_unwrap!(domain, LinuxErrno::EAFNOSUPPORT.into());
    let socket_type = SocketType::try_from(s_type & SOCKET_TYPE_MASK as usize);
    error_unwrap!(socket_type, LinuxErrno::EBADF.into());
    let task = current_task().unwrap();
    let socket = SocketData::new(domain, socket_type, protocol);
    error_unwrap!(socket, socket.err().unwrap().into());
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

/// 一个系统调用，用于绑定socket的地址和端口。
///
/// + `socketfd`: 指明要操作socket的文件描述符fd;
/// + `sockaddr`: 指明存储有关绑定信息([`RawIpV4Addr`])的地址;
/// + `len`: `address`([`RawIpV4Addr`])的长度。
///
/// 执行成功则返回0，否则返回错误信息。
#[syscall_func(200)]
pub fn bind(socketfd: usize, sockaddr: usize, len: usize) -> isize {
    let socket_fd = common_socket_syscall(socketfd);
    error_unwrap!(socket_fd, socket_fd.err().unwrap());
    let socket_addr = socket_addr_resolution(sockaddr, len);
    error_unwrap!(socket_addr, socket_addr.err().unwrap());
    let socket = socket_fd.get_socketdata_mut();
    match socket.bind(socket_addr.clone()) {
        Ok(()) => {
            let local_addr = socket.local_addr();
            warn!(
                "[{:?}] {:?} connect to ip: {:?}",
                socket.s_type, local_addr, socket_addr
            );
            0
        }
        Err(e) => e.into(),
    }
}

/// 一个系统调用，用于等待client提交连接请求，一般用于bind之后，accept之前
///
/// + `socketfd`: 指明要操作socket的文件描述符fd;
/// + `backlog`: 指明套接字侦听队列中正在处于半连接状态(等待accept)的请求数最大值。如果该值小于等于0，则自动调为0，同时也有最大值上限。
///
/// 执行成功则返回0，否则返回错误信息。
#[syscall_func(201)]
pub fn listening(socketfd: usize, backlog: usize) -> isize {
    let socket_fd = common_socket_syscall(socketfd);
    error_unwrap!(socket_fd, socket_fd.err().unwrap());
    let socket = socket_fd.get_socketdata();
    match socket.listening(backlog) {
        Ok(_) => {
            warn!("socket {:?} listening", socket.local_addr());
            0
        }
        Err(e) => e.into(),
    }
}

/// 一个系统调用，用于取出套接字listen队列中的第一个连接，创建一个与指定套接字具有相同套接字类型的地址族的新套接字。
/// 新套接字用于传递数据，原套接字继续处理侦听队列中的连接请求。如果侦听队列中无请求，accept()将阻塞。
///
/// + `socketfd`: 指明要操作socket的文件描述符fd，需经过bind()和listen()处理;
/// + `socket_addr`: 要么为空，要么指明保存accept成功的客户端相关信息([`RawIpV4Addr`])的地址;
/// + `addr_len`: 保存连接的client相关信息`address`长度的地址。
///
/// 执行成功则返回新的套接字的文件描述符，否则返回错误信息.
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
                warn!("accept peer addr: {:?}", peer_addr);
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

/// 一个系统调用，用于client请求在一个套接字上建立连接。
///
/// + `socketfd`: 指明要操作socket的文件描述符fd;
/// + `socket_addr`: 指明保存服务器地址和端口号的数据结构([`RawIpV4Addr`])的地址;
/// + `len`: `socket_addr`长度的地址。
///
/// 执行成功则返回0，否则返回错误信息。
///
/// Note: For netperf_test, the server may be run after client, so we nedd allow
/// client retry once
#[syscall_func(203)]
pub fn connect(socketfd: usize, socket_addr: usize, len: usize) -> isize {
    let socket_addr = socket_addr_resolution(socket_addr, len);
    error_unwrap!(socket_addr, socket_addr.err().unwrap());
    let socket_fd = common_socket_syscall(socketfd);
    error_unwrap!(socket_fd, socket_fd.err().unwrap());
    let socket = socket_fd.get_socketdata();
    let mut retry = 1;
    while retry >= 0 {
        match socket.connect(socket_addr.clone()) {
            Ok(_) => {
                let local_addr = socket.local_addr();
                warn!(
                    "[{:?}] {:?} connect to ip: {:?}",
                    socket.s_type, local_addr, socket_addr
                );
                return 0;
            }
            Err(e) => {
                if retry == 0 {
                    return e.into();
                }
                if e == LinuxErrno::EAGAIN {
                    return LinuxErrno::EINPROGRESS.into();
                }
                retry -= 1;
                do_suspend();
            }
        }
    }
    0
}

/// 一个系统调用，查询一个套接字本地bind()的相关信息。
///
/// + `socketfd`: 指明要操作socket的文件描述符fd;
/// + `socket_addr`: 指明相关信息([`RawIpV4Addr`])将要保存的地址;
/// + `len`: 保存`address`长度的地址。
///
/// 执行成功则返回0，否则返回错误信息。
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

/// 一个系统调用，用于获取一个本地套接字所连接的远程服务器的信息。
///
/// + `socketfd`: 指明要操作socket的文件描述符fd;
/// + `socket_addr`: 指明连接的客户端相关信息([`RawIpV4Addr`])将要保存的地址;
/// + `len`: 保存`address`长度的地址。
///
/// 执行成功则返回0，否则返回错误信息。
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

/// 一个系统调用，用于发送消息。当面向连接时，dest_addr被忽略；当非面向连接时，消息发送给dest_addr。
///
/// + `socketfd`: 指明要操作socket的文件描述符fd;
/// + `message`: 指明要发送的消息的首地址;
/// + `length`: 指明`message`的长度;
/// + `flags`: 指明发送操作的类型;
/// + `dest_addr`: 指明保存目的地的相关信息([`RawIpV4Addr`])的地址;
/// + `dest_len`: 指明`dest_addr`([`RawIpV4Addr`])的大小。
///
/// 如果发送成功，返回发送的字节数；否则返回错误信息.
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
        _ => {}
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
    do_suspend();
    send as isize
}

/// 一个系统调用，用于接收消息。消息源地址的相关信息将会保存在src_addr所指向的位置处。
///
/// + `socketfd`: 指明要操作socket的文件描述符fd;
/// + `buffer`: 指明接收消息的缓冲区的首地址;
/// + `length`: 指明缓冲区的长度(能接收消息的最大长度);
/// + `flags`: 指明接收操作的类型;
/// + `src_addr`: 指明消息源地址的相关信息([`RawIpV4Addr`])的保存地址。当该值为空时，不进行相关信息的保存;
/// + `addr_len`: 指明`src_addr`([`RawIpV4Addr`])大小的保存地址。
///
/// 如果接收成功，返回接收message的字节数；否则返回错误信息。
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

/// (待完成)一个系统调用函数，用于设置套接字的选项。
///
/// + `socketfd`: 指明要操作socket的文件描述符fd;
/// + `level`: 定义选项的级别，包括`Ip`，`Socket`，`TCP`等，详情可见[`SocketLevel`];
/// + `opt_name`: 在对应level下，为其设置值的套接字选项，详情可见[`SocketOption`];
/// + `_opt_value`: 存储选项值位置的指针;
/// + `_opt_len`: 选项值长度;
///
/// 如果函数执行成功，则返回0；否则返回错误信息。
#[syscall_func(208)]
pub fn setsockopt(
    socketfd: usize,
    level: usize,
    opt_name: usize,
    _opt_value: usize,
    _opt_len: u32,
) -> isize {
    let socket_fd = common_socket_syscall(socketfd);
    error_unwrap!(socket_fd, socket_fd.err().unwrap());
    let _socket = socket_fd.get_socketdata();
    let level = SocketLevel::try_from(level);
    error_unwrap!(level, LinuxErrno::EINVAL.into());
    match level {
        SocketLevel::Ip => {}
        SocketLevel::Socket => {
            let opt_name = SocketOption::try_from(opt_name);
            error_unwrap!(opt_name, LinuxErrno::EINVAL.into());
            warn!("[setsockopt] level: {:?}, opt_name: {:?}", level, opt_name);
        }
        SocketLevel::Tcp => {
            let opt_name = TcpSocketOption::try_from(opt_name);
            error_unwrap!(opt_name, LinuxErrno::EINVAL.into());
            warn!("[setsockopt] level: {:?}, opt_name: {:?}", level, opt_name);
        }
    }
    0
}

/// 一个系统调用函数，用于获取套接字的选项。
///
/// + `socketfd`: 指明要操作socket的文件描述符fd;
/// + `level`: 定义选项的级别，包括`Ip`，`Socket`，`TCP`等，详情可见[`SocketLevel`];
/// + `opt_name`: 在对应level下，要为其检索值的套接字选项，详情可见[`SocketOption`];
/// + `opt_value`: 一个指向将要保存请求选项值的缓冲区的指针;
/// + `_opt_len`: 指向保存选项值长度的指针;
///
/// 如果函数执行成功，则返回0；否则返回错误信息。
#[syscall_func(209)]
pub fn getsockopt(
    socketfd: usize,
    level: usize,
    opt_name: usize,
    opt_value: usize,
    opt_len: usize,
) -> isize {
    let socket_fd = common_socket_syscall(socketfd);
    error_unwrap!(socket_fd, socket_fd.err().unwrap());
    let _socket = socket_fd.get_socketdata();
    let level = SocketLevel::try_from(level);
    error_unwrap!(level, LinuxErrno::EINVAL.into());
    match level {
        SocketLevel::Ip => {}
        SocketLevel::Socket => {
            let opt_name = SocketOption::try_from(opt_name);
            error_unwrap!(opt_name, LinuxErrno::EINVAL.into());
            warn!("[getsockopt] level: {:?}, opt_name: {:?}", level, opt_name);
            match opt_name {
                SocketOption::SO_RCVBUF => {
                    let opt_value_ref = current_task()
                        .unwrap()
                        .access_inner()
                        .transfer_raw_ptr_mut(opt_value as *mut u32);
                    *opt_value_ref = simple_net::common::SOCKET_RECV_BUFFER_SIZE as u32;
                }
                SocketOption::SO_SNDBUF => {
                    let opt_value_ref = current_task()
                        .unwrap()
                        .access_inner()
                        .transfer_raw_ptr_mut(opt_value as *mut u32);
                    *opt_value_ref = simple_net::common::SOCKET_SEND_BUFFER_SIZE as u32;
                }
                SocketOption::SO_ERROR => {
                    let opt_value_ref = current_task()
                        .unwrap()
                        .access_inner()
                        .transfer_raw_ptr_mut(opt_value as *mut u32);
                    *opt_value_ref = 0;
                }
                _ => {}
            }
            let opt_len_ref = current_task()
                .unwrap()
                .access_inner()
                .transfer_raw_ptr_mut(opt_len as *mut u32);
            *opt_len_ref = core::mem::size_of::<u32>() as u32;
        }
        SocketLevel::Tcp => {
            let opt_name = TcpSocketOption::try_from(opt_name);
            error_unwrap!(opt_name, LinuxErrno::EINVAL.into());
            warn!("[getsockopt] level: {:?}, opt_name: {:?}", level, opt_name);
            match opt_name {
                TcpSocketOption::TCP_MAXSEG => {
                    let opt_value_ref = current_task()
                        .unwrap()
                        .access_inner()
                        .transfer_raw_ptr_mut(opt_value as *mut u32);
                    *opt_value_ref = simple_net::common::MAX_SEGMENT_SIZE as u32;
                }
                TcpSocketOption::TCP_NODELAY => {
                    let opt_value_ref = current_task()
                        .unwrap()
                        .access_inner()
                        .transfer_raw_ptr_mut(opt_value as *mut u32);
                    *opt_value_ref = 0;
                }
                _ => {}
            }
        }
    }
    0
}

/// 一个系统调用，用于关闭一个socket的发送操作或接收操作。
///
/// + `socketfd`: 指明要操作socket的文件描述符fd;
/// + `how`: 指明要关闭的操作：包括只关闭Read，只关闭Write，RW都关闭，相关Flag值可见[`ShutdownFlag`]。
///
/// 执行成功则返回0，否则返回错误信息。
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

/// (待实现)一个系统调用，创建一对未绑定的socket套接字，该对套接字可以用于全双工通信，或者用于父子进程之间的通信。
///
/// 如果向其中的一个socket写入后，再从该socket读时，就会发生阻塞。只能在另一个套接字中读。往往和shutdown()配合使用
///
/// + `domain`: 指明套接字被创建的协议簇(包括文件路径协议簇和网络地址协议簇，具体可见[`Domain`]);
/// + `type`: 指明被创建的socket的类型，具体可见[`SocketType`];
/// + `protocol`: 指明该socket应用于某一个特定的协议上。当确定了套接字使用的协议簇和类型，该参数可以取为0。
/// + `sv[2]`:  用于存放一对套接字的文件描述符。
///
/// 如果创建成功则返回0，否则返回错误信息。
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

/// 通过socket文件描述符fd获取对应的文件
fn common_socket_syscall(sockfd: usize) -> Result<Arc<KFile>, isize> {
    let task = current_task().unwrap();
    let socket_fd = task.get_file(sockfd);
    option_unwrap!(socket_fd, Err(LinuxErrno::EBADF.into()));
    Ok(socket_fd)
}
