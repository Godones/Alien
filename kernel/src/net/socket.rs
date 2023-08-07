use alloc::boxed::Box;
use alloc::sync::Arc;
use core::fmt::{Debug, Formatter};
use core::net::SocketAddr;

use rvfs::dentry::DirEntry;
use rvfs::file::{File, FileExtOps, FileMode, FileOps, OpenFlags};
use rvfs::mount::VfsMount;
use rvfs::superblock::{DataOps, Device};
use rvfs::StrResult;

use simple_net::tcp::TcpSocket;
use simple_net::udp::UdpSocket;
use syscall_define::net::{Domain, SocketType};
use syscall_define::LinuxErrno;

use crate::net::addr::SocketAddrExt;
use crate::net::port::neterror2linux;

use super::ShutdownFlag;

#[derive(Debug)]
pub struct SocketData {
    /// socket 通信域  
    pub domain: Domain,
    /// 连接类型
    pub s_type: SocketType,
    /// 具体的通信协议
    pub protocol: usize,
    pub socket: Socket,
}

pub enum Socket {
    Tcp(TcpSocket),
    Udp(UdpSocket),
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
    pub fn new(domain: Domain, s_type: SocketType, protocol: usize) -> Arc<File> {
        let raw_socket = match domain {
            Domain::AF_UNIX => {
                panic!("AF_UNIX is not supported")
            }
            Domain::AF_INET => match s_type {
                SocketType::SOCK_STREAM => Socket::Tcp(TcpSocket::new()),
                SocketType::SOCK_DGRAM => Socket::Udp(UdpSocket::new()),
                _ => {
                    panic!("unsupported socket type: {:?}", s_type)
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
        file.f_dentry.access_inner().d_inode.access_inner().data = Some(socket_data);
        Arc::new(file)
    }

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
        file.f_dentry.access_inner().d_inode.access_inner().data = Some(socket_data);
        Arc::new(file)
    }
    pub fn socket_type(&self) -> SocketType {
        self.s_type
    }

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

    pub fn is_tcp(&self) -> bool {
        match self.s_type {
            SocketType::SOCK_STREAM => true,
            _ => false,
        }
    }

    pub fn is_udp(&self) -> bool {
        match self.s_type {
            SocketType::SOCK_DGRAM => true,
            _ => false,
        }
    }

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

    pub fn accept(&self) -> Result<Arc<File>, LinuxErrno> {
        match &self.socket {
            Socket::Tcp(tcp) => tcp
                .accept()
                .map(|socket| Ok(self.new_connected(socket)))
                .map_err(neterror2linux)?,
            _ => Err(LinuxErrno::EOPNOTSUPP),
        }
    }

    pub fn listening(&self, _back_log: usize) -> Result<(), LinuxErrno> {
        match &self.socket {
            Socket::Tcp(tcp) => tcp.listen().map_err(neterror2linux),
            _ => Err(LinuxErrno::EOPNOTSUPP),
        }
    }

    pub fn connect(&self, ip: SocketAddrExt) -> Result<(), LinuxErrno> {
        match &self.socket {
            Socket::Tcp(tcp) => {
                tcp.connect(ip.get_socketaddr()).map_err(neterror2linux)?;
            }
            Socket::Udp(udp) => {
                udp.connect(ip.get_socketaddr()).map_err(neterror2linux)?;
            }
            _ => {
                panic!("bind is not supported")
            }
        }
        Ok(())
    }
    pub fn send_to(
        &self,
        message: &[u8],
        _flags: usize,
        dest_addr: Option<SocketAddrExt>,
    ) -> Result<usize, LinuxErrno> {
        match &self.socket {
            Socket::Tcp(tcp) => tcp.send(message).map_err(neterror2linux),
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
                let recv = udp.recv(message).map_err(neterror2linux)?;
                let peer_addr = udp.peer_addr().map_err(neterror2linux)?;
                Ok((recv, peer_addr))
            }
            _ => {
                panic!("bind is not supported")
            }
        }
    }

    pub fn shutdown(&self, _sdflag: ShutdownFlag) -> Result<(), LinuxErrno> {
        match &self.socket {
            Socket::Tcp(tcp) => tcp.shutdown().map_err(neterror2linux),
            Socket::Udp(udp) => udp.shutdown().map_err(neterror2linux),
            _ => {
                panic!("bind is not supported")
            }
        }
    }

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
            _ => {
                panic!("bind is not supported")
            }
        }
    }

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

    pub fn ready_read(&self) -> bool {
        match &self.socket {
            Socket::Tcp(tcp) => {
                let res = tcp.poll();
                if let Ok(res) = res {
                    res.readable
                } else {
                    false
                }
            }
            Socket::Udp(udp) => {
                let res = udp.poll();
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

fn socket_file_release(file: Arc<File>) -> StrResult<()> {
    error!("socket file release");
    let dentry_inner = file.f_dentry.access_inner();
    let inode_inner = dentry_inner.d_inode.access_inner();
    let data = inode_inner.data.as_ref().unwrap();
    let data = SocketData::from_ptr(data.data());
    data.socket = Socket::None;
    Ok(())
}

fn socket_ready_to_read(file: Arc<File>) -> bool {
    let dentry_inner = file.f_dentry.access_inner();
    let inode_inner = dentry_inner.d_inode.access_inner();
    let data = inode_inner.data.as_ref().unwrap();
    let socket = SocketData::from_ptr(data.data());
    socket.ready_read()
}

fn socket_ready_to_write(file: Arc<File>) -> bool {
    let dentry_inner = file.f_dentry.access_inner();
    let inode_inner = dentry_inner.d_inode.access_inner();
    let data = inode_inner.data.as_ref().unwrap();
    let socket = SocketData::from_ptr(data.data());
    socket.ready_write()
}
