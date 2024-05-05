use alloc::{
    string::{String, ToString},
    vec,
};
use core::net::{IpAddr, Ipv4Addr, SocketAddr};

use constants::{net::Domain, AlienResult, LinuxErrno};
use knet::addr::{RawIpV4Addr, SocketAddrExt};

use crate::task::current_task;

/// 地址解析，将根据`family_user_addr`的[`Domain`]类型分类进行解析。
///
/// 对于`AF_INET`将解析成SocketAddrExt::SocketAddr(SocketAddr)，
/// 对于`AF_UNIX`将解析成ocketAddrExt::LocalPath(String)，详情可见[`SocketAddrExt`]。
pub fn socket_addr_resolution(family_user_addr: usize, len: usize) -> AlienResult<SocketAddrExt> {
    let task = current_task().unwrap();
    let family = task
        .access_inner()
        .transfer_raw_ptr(family_user_addr as *const u16);
    let domain = Domain::try_from(*family as usize).map_err(|_| LinuxErrno::EINVAL)?;
    match domain {
        Domain::AF_INET => {
            let mut ip_addr = RawIpV4Addr::default();
            task.access_inner()
                .copy_from_user(family_user_addr as *const RawIpV4Addr, &mut ip_addr);
            let ip = u32::from_be_bytes(ip_addr.addr.to_le_bytes());
            let ipv4_addr = IpAddr::V4(Ipv4Addr::from(ip));
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
