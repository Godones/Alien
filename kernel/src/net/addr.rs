use alloc::string::{String, ToString};
use alloc::vec;
use core::fmt::{Debug, Formatter};

use syscall_define::net::Domain;
use syscall_define::LinuxErrno;

use crate::task::current_task;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum IpAddr {
    /// ip 地址+端口
    Ipv4(u32, u16),
    Ipv6(u128, u16),
    LocalPath(String),
    /// 初始化
    Empty,
    /// 未知
    Unknown,
}

impl Debug for IpAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            IpAddr::Ipv4(ip, port) => {
                let ipv4 = core::net::Ipv4Addr::from(*ip);
                let socket = core::net::SocketAddr::new(core::net::IpAddr::V4(ipv4), *port);
                f.write_fmt(format_args!("{:?}", socket))
            }
            IpAddr::Ipv6(ip, port) => {
                let ipv6 = core::net::Ipv6Addr::from(*ip);
                let socket = core::net::SocketAddr::new(core::net::IpAddr::V6(ipv6), *port);
                f.write_fmt(format_args!("{:?}", socket))
            }
            IpAddr::LocalPath(path) => f.write_fmt(format_args!("local path:{}", path)),
            IpAddr::Empty => f.write_fmt(format_args!("empty")),
            IpAddr::Unknown => f.write_fmt(format_args!("unknown")),
        }
    }
}

impl IpAddr {
    pub fn port(&self) -> Option<u16> {
        match self {
            IpAddr::Ipv4(_, port) => Some(*port),
            IpAddr::Ipv6(_, port) => Some(*port),
            _ => None,
        }
    }

    pub fn to_be_ipv4(&self) -> Option<IpV4Addr> {
        match self {
            IpAddr::Ipv4(ip, port) => Some(IpV4Addr {
                family: Domain::AF_INET as u16,
                port: port.to_be(),
                addr: ip.to_be(),
                zero: [0; 8],
            }),
            _ => None,
        }
    }
    pub fn is_valid(&self) -> bool {
        match self {
            IpAddr::Empty => false,
            IpAddr::Unknown => false,
            _ => true,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct IpV4Addr {
    pub family: u16,
    pub port: u16,
    pub addr: u32,
    pub zero: [u8; 8],
}

impl IpV4Addr {
    pub fn to_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self as *const IpV4Addr as *const u8,
                core::mem::size_of::<IpV4Addr>(),
            )
        }
    }
}

// 地址解析
pub fn socket_addr_resolution(family_user_addr: *const u16, len: usize) -> Result<IpAddr, isize> {
    let task = current_task().unwrap();
    let family = task.access_inner().transfer_raw_ptr(family_user_addr);
    let domain = Domain::try_from(*family as usize);
    if domain.is_err() {
        return Err(LinuxErrno::EINVAL.into());
    }
    match domain.unwrap() {
        Domain::AF_INET => {
            let mut ip_addr = IpV4Addr::default();
            task.access_inner()
                .copy_from_user(family_user_addr as *const IpV4Addr, &mut ip_addr);
            Ok(IpAddr::Ipv4(
                u32::from_be(ip_addr.addr),
                u16::from_be(ip_addr.port),
            ))
        }
        Domain::AF_UNIX => {
            // local path
            let mut buf = vec![0u8; len];
            task.access_inner().copy_from_user_buffer(
                family_user_addr as *const u8,
                buf.as_mut_ptr(),
                len,
            );
            let path = String::from_utf8_lossy(&buf[2..len - 2]).to_string();
            Ok(IpAddr::LocalPath(path))
        }
    }
}

pub fn split_u32_to_u8s(num: u32) -> [u8; 4] {
    let byte1 = (num >> 24) as u8;
    let byte2 = (num >> 16) as u8;
    let byte3 = (num >> 8) as u8;
    let byte4 = num as u8;

    [byte1, byte2, byte3, byte4]
}
