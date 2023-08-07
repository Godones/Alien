use alloc::string::{String, ToString};
use alloc::vec;
use core::fmt::Debug;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};

use syscall_define::net::Domain;
use syscall_define::LinuxErrno;

use crate::error_unwrap;
use crate::task::current_task;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum SocketAddrExt {
    LocalPath(String),
    SocketAddr(SocketAddr),
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct RawIpV4Addr {
    pub family: u16,
    pub port: u16,
    pub addr: u32,
    pub zero: [u8; 8],
}

impl SocketAddrExt {
    pub fn get_socketaddr(&self) -> SocketAddr {
        match self {
            SocketAddrExt::LocalPath(_) => {
                panic!("Can't get socketaddr from local path")
            }
            SocketAddrExt::SocketAddr(addr) => *addr,
        }
    }
}

impl From<SocketAddr> for RawIpV4Addr {
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
        let ip = u32::from_be_bytes(ip);
        Self {
            family: Domain::AF_INET as u16,
            port: port.to_be(),
            addr: ip,
            zero: [0u8; 8],
        }
    }
}

// 地址解析
pub fn socket_addr_resolution(family_user_addr: usize, len: usize) -> Result<SocketAddrExt, isize> {
    let task = current_task().unwrap();
    let family = task
        .access_inner()
        .transfer_raw_ptr(family_user_addr as *const u16);
    let domain = Domain::try_from(*family as usize);
    error_unwrap!(domain, Err(LinuxErrno::EINVAL.into()));
    match domain {
        Domain::AF_INET => {
            let mut ip_addr = RawIpV4Addr::default();
            task.access_inner()
                .copy_from_user(family_user_addr as *const RawIpV4Addr, &mut ip_addr);
            let ipv4_addr = IpAddr::V4(Ipv4Addr::from(ip_addr.addr));
            let port = u16::from_be(ip_addr.port);
            Ok(SocketAddrExt::SocketAddr(SocketAddr::new(
                ipv4_addr.into(),
                port,
            )))
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
            Ok(SocketAddrExt::LocalPath(path))
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
