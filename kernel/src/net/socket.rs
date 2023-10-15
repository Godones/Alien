//! Alien 内核中使用的套接字结构。
//!
//! Alien 目前采用 [`SocketData`] 存储每一个套接字的相关信息，其字段 socket (结构为 [`Socket`]) 存储套接字的具体信息，
//! 对于套接字的操作最终都会归结于对 [`SocketData`] 的操作。
//!
//! 套接字的创建时，需要返回一个创建的套接字文件描述符用于获取和操作该套接字。依据 Alien 所使用的 rvfs 中对文件 `File`
//! 的规定，我们只需为套接字文件规定好 [`socket_file_release`]、[`socket_file_write`]、[`socket_file_read`]、
//! [`socket_ready_to_read`]、[`socket_ready_to_write`] 几个操作函数，即可快速的创建套接字文件，并将其放入进程的文件描述
//! 符表中，具体有关套接字文件的创建，可见 [`SocketData::new`] 的实现。
use alloc::boxed::Box;
use alloc::sync::Arc;
use core::fmt::{Debug, Formatter};
use core::net::SocketAddr;

use rvfs::dentry::DirEntry;
use rvfs::file::{File, FileExtOps, FileMode, FileOps, OpenFlags};
use rvfs::inode::SpecialData;
use rvfs::mount::VfsMount;
use rvfs::superblock::{DataOps, Device};
use rvfs::StrResult;

use pconst::net::{Domain, SocketType};
use pconst::LinuxErrno;
use simple_net::tcp::TcpSocket;
use simple_net::udp::UdpSocket;

use crate::net::addr::SocketAddrExt;
use crate::net::port::neterror2linux;
use crate::net::unix::UnixSocket;
use crate::task::do_suspend;

use super::ShutdownFlag;

/// Alien 内核中对于每一个套接字所存储的相关信息。所有系统调用最后都要归到该结构的操作。
#[derive(Debug)]
pub struct SocketData {
    /// socket 通信域  
    pub domain: Domain,
    /// 连接类型
    pub s_type: SocketType,
    /// 具体的通信协议
    pub protocol: usize,
    /// 具体的套接字数据，具体可见 [`Socket`]
    pub socket: Socket,
}

/// 用于记录一个套接字的具体数据。
///
/// 针对套接字类型，`Tcp` 和 `Udp` 类型中存储的具体数据是 `simple_net` 中的 [`TcpSocket`] 和 [`UdpSocket`] 类型；
/// `Unix` 类型中存储的数据是 [`UnixSocket`]。
pub enum Socket {
    Tcp(TcpSocket),
    Udp(UdpSocket),
    Unix(UnixSocket),
    None,
}

impl Debug for Socket {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Socket::Tcp(_) => {
                write!(f, "Tcp")
            }
            Socket::Udp(_) => {
                write!(f, "Udp")
            }
            Socket::None => {
                write!(f, "None")
            }
            Socket::Unix(_) => {
                write!(f, "Unix")
            }
        }
    }
}

impl DataOps for SocketData {
    fn device(&self, _name: &str) -> Option<Arc<dyn Device>> {
        None
    }
    fn data(&self) -> *const u8 {
        self as *const _ as *const u8
    }
}

impl SocketData {
    pub fn from_ptr(ptr: *const u8) -> &'static mut Self {
        unsafe { &mut *(ptr as *mut Self) }
    }

    /// 用于创建一个新的套接字数据 `SocketData` 结构，返回创建的文件描述符。一般被系统调用 [`socket`] 所调用。
    ///
    /// 执行过程中会创建一个对应的套接字文件，打开后将对应的文件描述符放入进程的文件描述符表中，
    /// 对于 `Alien` 中对文件的相关定义可见 `rvfs` 模块中的 `File` 结构。这里对套接字文件的相关
    /// 操作函数可见 [`socket_file_release`]、[`socket_file_write`]、[`socket_file_read`]、
    /// [`socket_ready_to_read`]、[`socket_ready_to_write`]。
    pub fn new(
        domain: Domain,
        s_type: SocketType,
        protocol: usize,
    ) -> Result<Arc<File>, LinuxErrno> {
        let raw_socket = match domain {
            Domain::AF_UNIX => {
                error!("AF_UNIX is not supported");
                Socket::Unix(UnixSocket::new())
            }
            Domain::AF_INET => match s_type {
                SocketType::SOCK_STREAM => Socket::Tcp(TcpSocket::new()),
                SocketType::SOCK_DGRAM => Socket::Udp(UdpSocket::new()),
                _ => {
                    error!("unsupported socket type: {:?}", s_type);
                    return Err(LinuxErrno::EPROTONOSUPPORT);
                }
            },
        };

        let socket_data = Self {
            domain,
            s_type,
            protocol,
            socket: raw_socket,
        };

        let socket_data = Box::new(socket_data);
        let mut file_ops = FileOps::empty();
        file_ops.release = socket_file_release;
        file_ops.write = socket_file_write;
        file_ops.read = socket_file_read;
        let file = File::new(
            Arc::new(DirEntry::empty()),
            Arc::new(VfsMount::empty()),
            OpenFlags::O_RDWR,
            FileMode::FMODE_RDWR,
            file_ops,
        );
        file.access_inner().f_ops_ext = {
            let mut file_ext_ops = FileExtOps::empty();
            file_ext_ops.is_ready_read = socket_ready_to_read;
            file_ext_ops.is_ready_write = socket_ready_to_write;
            file_ext_ops
        };
        file.f_dentry
            .access_inner()
            .d_inode
            .access_inner()
            .special_data = Some(SpecialData::Socket);
        file.f_dentry.access_inner().d_inode.access_inner().data = Some(socket_data);
        Ok(Arc::new(file))
    }

    /// 用于对一个已经存在的 tcp_socket 创建对应的套接字文件。一般在 accept 成功接受一个 client 后被调用。
    ///
    /// 对于 `Alien` 中对文件的相关定义可见 `rvfs` 模块中的 `File` 结构。
    fn new_connected(&self, tcp_socket: TcpSocket) -> Arc<File> {
        let socket_data = Self {
            domain: self.domain,
            s_type: self.s_type,
            protocol: self.protocol,
            socket: Socket::Tcp(tcp_socket),
        };
        let socket_data = Box::new(socket_data);
        let mut file_ops = FileOps::empty();
        file_ops.release = socket_file_release;
        file_ops.write = socket_file_write;
        file_ops.read = socket_file_read;
        let file = File::new(
            Arc::new(DirEntry::empty()),
            Arc::new(VfsMount::empty()),
            OpenFlags::O_RDWR,
            FileMode::FMODE_RDWR,
            file_ops,
        );
        file.access_inner().f_ops_ext = {
            let mut file_ext_ops = FileExtOps::empty();
            file_ext_ops.is_ready_read = socket_ready_to_read;
            file_ext_ops.is_ready_write = socket_ready_to_write;
            file_ext_ops
        };
        file.f_dentry
            .access_inner()
            .d_inode
            .access_inner()
            .special_data = Some(SpecialData::Socket);
        file.f_dentry.access_inner().d_inode.access_inner().data = Some(socket_data);
        Arc::new(file)
    }

    /// 返回套接字的类型
    pub fn socket_type(&self) -> SocketType {
        self.s_type
    }

    /// 设置套接字的阻塞状态。用于传入 SOCK_NONBLOCK 标志位的套接字创建过程中。
    pub fn set_socket_nonblock(&self, blocking: bool) {
        match &self.socket {
            Socket::Tcp(tcp) => {
                tcp.set_nonblocking(blocking);
            }
            Socket::Udp(udp) => {
                udp.set_nonblocking(blocking);
            }
            _ => {
                panic!("set_socket_nonblock is not supported")
            }
        }
    }

    /// 用于绑定套接字端口，tcp 和 udp 可用。被系统调用 [`bind`] 调用。
    pub fn bind(&self, socket_addr: SocketAddrExt) -> Result<(), LinuxErrno> {
        match &self.socket {
            Socket::Tcp(tcp) => {
                tcp.bind(socket_addr.get_socketaddr())
                    .map_err(neterror2linux)?;
            }
            Socket::Udp(udp) => {
                udp.bind(socket_addr.get_socketaddr())
                    .map_err(neterror2linux)?;
            }
            _ => {
                panic!("bind is not supported")
            }
        }
        Ok(())
    }

    /// 用于处理一个 client 的连接请求，仅限于 Tcp 套接字。被系统调用 [`accept`] 调用。
    ///
    /// 如果该套接字不是 Tcp 套接字，将直接返回 Err。
    pub fn accept(&self) -> Result<Arc<File>, LinuxErrno> {
        match &self.socket {
            Socket::Tcp(tcp) => tcp
                .accept()
                .map(|socket| Ok(self.new_connected(socket)))
                .map_err(neterror2linux)?,
            _ => Err(LinuxErrno::EOPNOTSUPP),
        }
    }

    /// 用于监听一个端口，仅限于 Tcp 套接字。被系统调用 [`listening`] 调用。
    ///
    /// 如果该套接字不是 Tcp 套接字，将直接返回 Err。
    pub fn listening(&self, _back_log: usize) -> Result<(), LinuxErrno> {
        match &self.socket {
            Socket::Tcp(tcp) => tcp.listen().map_err(neterror2linux),
            _ => Err(LinuxErrno::EOPNOTSUPP),
        }
    }

    /// 用于连接一个套接字。被系统调用 [`connect`] 调用。
    pub fn connect(&self, ip: SocketAddrExt) -> Result<(), LinuxErrno> {
        match &self.socket {
            Socket::Tcp(tcp) => {
                tcp.connect(ip.get_socketaddr()).map_err(neterror2linux)?;
            }
            Socket::Udp(udp) => {
                udp.connect(ip.get_socketaddr()).map_err(neterror2linux)?;
            }
            Socket::Unix(unix) => unix.connect(ip.get_local_path())?,
            _ => {
                panic!("bind is not supported")
            }
        }
        Ok(())
    }

    /// 用于向一个套接字中发送消息。发送成功则返回发送的消息长度。被系统调用 [`sendto`] 调用。
    pub fn send_to(
        &self,
        message: &[u8],
        _flags: usize,
        dest_addr: Option<SocketAddrExt>,
    ) -> Result<usize, LinuxErrno> {
        match &self.socket {
            Socket::Tcp(tcp) => tcp.send(message).map_err(|x| neterror2linux(x)),
            Socket::Udp(udp) => {
                if let Some(dest_addr) = dest_addr {
                    udp.send_to(message, dest_addr.get_socketaddr())
                        .map_err(neterror2linux)
                } else {
                    udp.send(message).map_err(neterror2linux)
                }
            }
            _ => {
                panic!("bind is not supported")
            }
        }
    }

    /// 用于从一个套接字中接收消息，接收成功则返回接受的消息长度。被系统调用 [`recvfrom`] 调用。
    pub fn recvfrom(
        &self,
        message: &mut [u8],
        _flags: usize,
    ) -> Result<(usize, SocketAddr), LinuxErrno> {
        match &self.socket {
            Socket::Tcp(tcp) => {
                let recv = tcp.recv(message).map_err(neterror2linux)?;
                let peer_addr = tcp.peer_addr().map_err(neterror2linux)?;
                Ok((recv, peer_addr))
            }
            Socket::Udp(udp) => {
                let recv = udp.recv_from(message).map_err(neterror2linux)?;
                // let peer_addr = udp.peer_addr().map_err(neterror2linux)?;
                Ok((recv.0, recv.1))
            }
            _ => {
                panic!("bind is not supported")
            }
        }
    }

    /// 用于关闭套接字的读功能或写功能。被系统调用 [`shutdown`] 调用。
    pub fn shutdown(&self, _sdflag: ShutdownFlag) -> Result<(), LinuxErrno> {
        match &self.socket {
            Socket::Tcp(tcp) => tcp.shutdown().map_err(neterror2linux),
            Socket::Udp(udp) => udp.shutdown().map_err(neterror2linux),
            _ => {
                panic!("bind is not supported")
            }
        }
    }

    /// 用于获取当前套接字绑定的本地套接字地址信息。
    pub fn local_addr(&self) -> Option<SocketAddr> {
        match &self.socket {
            Socket::Tcp(tcp) => {
                let local_addr = tcp.local_addr();
                if let Ok(addr) = local_addr {
                    Some(addr)
                } else {
                    None
                }
            }
            Socket::Udp(udp) => {
                let local_addr = udp.local_addr();
                if let Ok(addr) = local_addr {
                    Some(addr)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// 用于获取当前套接字连接的远程服务器的套接字地址信息。
    pub fn peer_addr(&self) -> Option<SocketAddr> {
        match &self.socket {
            Socket::Tcp(tcp) => {
                let peer_addr = tcp.peer_addr();
                if let Ok(addr) = peer_addr {
                    Some(addr)
                } else {
                    None
                }
            }
            Socket::Udp(udp) => {
                let peer_addr = udp.peer_addr();
                if let Ok(addr) = peer_addr {
                    Some(addr)
                } else {
                    None
                }
            }
            _ => {
                panic!("bind is not supported")
            }
        }
    }

    /// 用于获取当前套接字是否有消息需要接收。
    pub fn ready_read(&self) -> bool {
        match &self.socket {
            Socket::Tcp(tcp) => {
                let res = tcp.poll();
                warn!("Tcp ready_read: {:?}", res);
                if let Ok(res) = res {
                    res.readable
                } else {
                    false
                }
            }
            Socket::Udp(udp) => {
                let res = udp.poll();
                warn!("Udp ready_read: {:?}", res);
                if let Ok(res) = res {
                    res.readable
                } else {
                    false
                }
            }
            _ => {
                panic!("bind is not supported")
            }
        }
    }

    ///用于获取当前套接字是否有消息需要发送。
    pub fn ready_write(&self) -> bool {
        match &self.socket {
            Socket::Tcp(tcp) => {
                let res = tcp.poll();
                if let Ok(res) = res {
                    res.writable
                } else {
                    false
                }
            }
            Socket::Udp(udp) => {
                let res = udp.poll();
                if let Ok(res) = res {
                    res.writable
                } else {
                    false
                }
            }
            _ => {
                panic!("bind is not supported")
            }
        }
    }
}

/// 套接字文件的释放操作，会使得对应的套接字数据的 socket 字段置为 (Socket::None)。
///
/// 有关文件操作的定义可见 `rvfs` 模块中的 `File` 结构。
fn socket_file_release(file: Arc<File>) -> StrResult<()> {
    error!("socket file release");
    let dentry_inner = file.f_dentry.access_inner();
    let inode_inner = dentry_inner.d_inode.access_inner();
    let data = inode_inner.data.as_ref().unwrap();
    let data = SocketData::from_ptr(data.data());
    data.socket = Socket::None;
    Ok(())
}

/// 套接字文件的读就绪操作，效果等同于 ready_read。当套接字中有未接收的数据时，会返回 true；否则返回 false。
///
/// 会调用 simple_net::poll_interfaces()。有关文件操作的定义可见 `rvfs` 模块中的 `File` 结构。
fn socket_ready_to_read(file: Arc<File>) -> bool {
    let dentry_inner = file.f_dentry.access_inner();
    let inode_inner = dentry_inner.d_inode.access_inner();
    let data = inode_inner.data.as_ref().unwrap();
    let socket = SocketData::from_ptr(data.data());
    simple_net::poll_interfaces();
    socket.ready_read()
}

/// 套接字文件的写就绪操作，效果等同于 ready_write。当套接字中有未发送的数据时，会返回 true；否则返回 false。
///
/// 会调用 simple_net::poll_interfaces()。有关文件操作的定义可见 `rvfs` 模块中的 `File` 结构。
fn socket_ready_to_write(file: Arc<File>) -> bool {
    let dentry_inner = file.f_dentry.access_inner();
    let inode_inner = dentry_inner.d_inode.access_inner();
    let data = inode_inner.data.as_ref().unwrap();
    let socket = SocketData::from_ptr(data.data());
    simple_net::poll_interfaces();
    socket.ready_write()
}

/// 套接字文件的写操作，效果等同于 send_to。
///
/// 有关文件操作的定义可见 `rvfs` 模块中的 `File` 结构。
fn socket_file_write(file: Arc<File>, buf: &[u8], _offset: u64) -> StrResult<usize> {
    let dentry_inner = file.f_dentry.access_inner();
    let inode_inner = dentry_inner.d_inode.access_inner();
    let data = inode_inner.data.as_ref().unwrap();
    let socket = SocketData::from_ptr(data.data());
    warn!("socket_file_write: {:?}", buf.len());
    simple_net::poll_interfaces();
    let res = socket.send_to(buf, 0, None).map_err(|x| {
        error!("socket_file_write: {:?}", x);
        match x {
            LinuxErrno::EAGAIN => "Try Again",
            _ => "Net Error",
        }
    });
    do_suspend();
    res
}

/// 套接字文件的读操作，效果等同于 recvfrom。
///
/// 有关文件操作的定义可见 `rvfs` 模块中的 `File` 结构。
fn socket_file_read(file: Arc<File>, buf: &mut [u8], _offset: u64) -> StrResult<usize> {
    let dentry_inner = file.f_dentry.access_inner();
    let inode_inner = dentry_inner.d_inode.access_inner();
    let data = inode_inner.data.as_ref().unwrap();
    let socket = SocketData::from_ptr(data.data());
    simple_net::poll_interfaces();
    let res = socket.recvfrom(buf, 0).map(|x| x.0).map_err(|x| {
        error!("socket_file_read: {:?}", x);
        match x {
            LinuxErrno::EAGAIN => "Try Again",
            _ => "Net Error",
        }
    });
    warn!("socket_file_read: {:?}, indeed {:?}", buf.len(), res);
    res
}
