use alloc::sync::Arc;

use core::mem::size_of;

use super::port::{read_from_port, write_to_port};
use super::addr::{Addr, LOCAL_LOOPBACK_ADDR, addr_resolution, IpAddr, };
use super::{SOCKET_WRTYPE, ShutdownFlag};


#[derive(Debug)]
pub struct Socket {
    /// socket 通信域  
    _domain: usize,
    /// 连接类型
    _s_type: usize,
    /// 具体的通信协议
    _protocol: usize,
    /// 连接的远端服务器的信息
    _peer_addr: Addr,
    /// 本地的信息
    _sock_addr: Addr,
    // 读写权限
    pub wr_type: SOCKET_WRTYPE,
}


impl Socket {
    pub fn new(_domain: usize, _s_type: usize, _protocol: usize) ->  Arc<Socket>{         
        Arc::new(Self{
            _domain,
            _s_type,
            _protocol,
            _peer_addr: Addr::Empty,
            _sock_addr: Addr::Empty,
            wr_type: SOCKET_WRTYPE::RDWR,
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



    pub fn shutdown(&self, sdflag: ShutdownFlag) -> isize {
        // match sdflag {
        //     ShutdownFlag::SHUTRD => {
        //         if self.wr_type == SOCKET_WRTYPE::RD_ONLY {
        //             self.wr_type = SOCKET_WRTYPE::CLOSE;
        //         } else {
        //             self.wr_type = SOCKET_WRTYPE::WR_ONLY;
        //         }
        //     },
        //     ShutdownFlag::SHUTWR => {
        //         if self.wr_type == SOCKET_WRTYPE::WR_ONLY {
        //             self.wr_type = SOCKET_WRTYPE::CLOSE;
        //         } else {
        //             self.wr_type = SOCKET_WRTYPE::RD_ONLY;
        //         }
        //     },
        //     ShutdownFlag::SHUTRDWR => {
        //         self.wr_type = SOCKET_WRTYPE::CLOSE;
        //     },
        // };
        0
    }
}