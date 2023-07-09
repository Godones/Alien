use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;


use crate::fs::FileLike;
use lazy_static::lazy_static;
use core::mem::size_of;

use super::port::{read_from_port, write_to_port};
use super::addr::{Addr, LOCAL_LOOPBACK_ADDR, addr_resolution, IpAddr, };

#[derive(Debug)]
pub struct Socket {
    /// socket 通信域  
    domain: usize,
    /// 连接类型
    s_type: usize,
    /// 具体的通信协议
    protocol: usize,
    /// 连接的远端服务器的信息
    peer_addr: Addr,
    /// 本地的信息
    sock_addr: Addr,
    // 读写权限
    // wr_type: 
}

/// socket放入文件描述符表，在FileLike枚举下
// const MAX_SOCKETS_NUM: usize = 512;
// lazy_static! {
//     static ref SOCKET_TABLE: Mutex<Vec<Option<Arc<Mutex<Socket>>>>> = 
//         unsafe { Mutex::new(Vec::with_capacity(MAX_SOCKETS_NUM))};
// }



impl Socket {
    pub fn new(domain: usize, s_type: usize, protocol: usize) ->  Arc<Socket>{         
        Arc::new(Self{
            domain,
            s_type,
            protocol,
            peer_addr: Addr::Empty,
            sock_addr: Addr::Empty,
        })
    }


    pub fn send_to(&self, message: &[u8], _flags: i32, dest_addr: *const usize) -> Option<usize> {
        match addr_resolution(dest_addr as *const u16) {
            Addr::Ipv4(ip, port) => {
                info!("send to ip {:x} port {}", ip, port);
                if ip == LOCAL_LOOPBACK_ADDR {
                    write_to_port(port, message)
                } else {
                    todo!()
                }
            }
            Addr::Empty => None,
            Addr::Unknown => None,
            
        }
    }


    pub fn recvfrom(&self, message: &mut [u8], _flags: i32, src_addr: * mut usize, src_len: *mut u32) -> Option<usize> {
        match addr_resolution(src_addr as *const u16) {
            Addr::Ipv4(ip, port) => {
                info!("receive from ip {:x} port {}", ip, port);
                // 按 syscall 描述，这里需要把地址信息的长度写到用户给的 src_len 的位置
                unsafe{ *src_len = size_of::<IpAddr>() as u32; }
                if ip == LOCAL_LOOPBACK_ADDR {
                    read_from_port(port, message)
                } else {
                    None
                }
            }
            Addr::Empty => None,
            Addr::Unknown => None,
        }
    }
}