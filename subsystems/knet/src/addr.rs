//! 在 Alien 内核中使用的 socket 套接字地址结构。
//!
//! Alien 中目前能够接收的套接字地址种类包括本地路径地址和网络套接字地址。
//! 对于从用户端传来的套接字地址，类似于 `linux` 中 `socket.h` 的套接字地址。
//! 大致结构如下:
//! + 2字节表明该套接字使用的地址协议族
//! + 2字节表明该套接字的端口
//! + 12字节的地址数据
//!
//! Alien 将会首先对传入的套接字的协议族进行解析，然后根据不同的地址协议族将其解析成 [`SocketAddrExt`] 结构，
//! 向下层的具体套接字中传递相应地址时，传递的也是 [`SocketAddrExt`] 结构。
//!
use alloc::string::String;
use core::{
    fmt::Debug,
    net::{IpAddr, SocketAddr},
};

use constants::net::Domain;

/// 用于存储套接字通信地址的结构，分为本地路径地址和网络套接字地址。
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum SocketAddrExt {
    LocalPath(String),
    SocketAddr(SocketAddr),
}

/// 用于存储一个Ipv4套接字相关信息的结构。对应 `linux` 中 `socket.h` 的 `sockaddr_in` 结构。
///
/// 在 socket 相关系统调用中，一般都先分析出套接字采用的地址协议族，如果是 `IPV4` 则会将传入的套接字相关信息解析成 `RawIpV4Addr`。
/// 且 `Alien` 目前默认使用网络套接字时，即采用 `IPV4` 协议。
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct RawIpV4Addr {
    /// 地址协议族
    pub family: u16,
    /// Ipv4 的端口
    pub port: u16,
    /// Ipv4 的地址
    pub addr: u32,
    /// 零位，用于后续扩展
    pub zero: [u8; 8],
}

impl SocketAddrExt {
    /// 获取网络套接字地址。当本结构中存储的是本地路径地址时，将导致 panic。
    pub fn get_socketaddr(&self) -> SocketAddr {
        match self {
            SocketAddrExt::LocalPath(_) => {
                panic!("Can't get socketaddr from local path")
            }
            SocketAddrExt::SocketAddr(addr) => *addr,
        }
    }

    /// 获取本地路径地址。当本结构中存储的是网络套接字地址时，将导致 panic。
    pub fn get_local_path(&self) -> String {
        match self {
            SocketAddrExt::LocalPath(path) => path.clone(),
            SocketAddrExt::SocketAddr(_) => {
                panic!("Can't get local path from socketaddr")
            }
        }
    }
}

impl From<SocketAddr> for RawIpV4Addr {
    /// 用一个 [`SocketAddr`] 结构 初始化 `RawIpV4Addr  `
    fn from(addr: SocketAddr) -> Self {
        let ip = addr.ip();
        let port = addr.port();
        let ip = match ip {
            IpAddr::V4(ip) => ip,
            IpAddr::V6(_) => {
                panic!("ipv6 is not supported")
            }
        };
        let ip = ip.octets();
        let ip = u32::from_le_bytes(ip);
        Self {
            family: Domain::AF_INET as u16,
            port: port.to_be(),
            addr: ip,
            zero: [0u8; 8],
        }
    }
}
