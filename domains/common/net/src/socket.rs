use alloc::{boxed::Box, sync::Arc};
use core::fmt::{Debug, Formatter};

use constants::{
    io::OpenFlags,
    net::{Domain, SocketType},
    AlienError, AlienResult,
};
use ksync::Mutex;
use log::error;
use netcore::{tcp::TcpSocket, udp::UdpSocket};

use crate::{socket, unix::UnixSocket};

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

impl SocketData {
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
    ) -> AlienResult<Arc<SocketFile>> {
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
                    return Err(AlienError::EPROTONOSUPPORT);
                }
            },
        };
        let socket_data = Self {
            domain,
            s_type,
            protocol,
            socket: raw_socket,
        };
        Ok(Arc::new(SocketFile::new(socket_data)))
    }
}

pub struct SocketFile {
    open_flag: Mutex<OpenFlags>,
    node: Mutex<Box<SocketData>>,
}

impl SocketFile {
    pub fn new(socket_data: SocketData) -> Self {
        Self {
            open_flag: Mutex::new(OpenFlags::O_RDWR),
            node: Mutex::new(Box::new(socket_data)),
        }
    }

    pub fn set_close_on_exec(&self) {
        *self.open_flag.lock() |= OpenFlags::O_CLOEXEC;
    }
}
