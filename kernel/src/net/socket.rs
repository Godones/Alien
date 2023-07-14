use alloc::boxed::Box;
use alloc::sync::Arc;
use core::mem::size_of;

use rvfs::dentry::DirEntry;
use rvfs::file::{File, FileMode, FileOps, OpenFlags};
use rvfs::mount::VfsMount;
use rvfs::superblock::{DataOps, Device};

use syscall_define::socket::{Domain, SocketType};

use super::addr::{addr_resolution, Addr, IpAddr, LOCAL_LOOPBACK_ADDR};
use super::port::{read_from_port, write_to_port};
use super::ShutdownFlag;

#[derive(Debug)]
pub struct SocketData {
    /// socket 通信域  
    domain: Domain,
    /// 连接类型
    s_type: SocketType,
    /// 具体的通信协议
    protocol: usize,
    /// 连接的远端服务器的信息
    peer_addr: Addr,
    /// 本地的信息
    sock_addr: Addr,
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
    pub fn new(domain: Domain, s_type: SocketType, protocol: usize) -> Arc<File> {
        let socket = Box::new(Self {
            domain,
            s_type,
            protocol,
            peer_addr: Addr::Empty,
            sock_addr: Addr::Empty,
        });
        let file = File::new(
            Arc::new(DirEntry::empty()),
            Arc::new(VfsMount::empty()),
            OpenFlags::O_RDWR,
            FileMode::FMODE_RDWR,
            FileOps::empty(),
        );
        file.f_dentry.access_inner().d_inode.access_inner().data = Some(socket);
        Arc::new(file)
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

    pub fn recvfrom(
        &self,
        message: &mut [u8],
        _flags: i32,
        src_addr: *mut usize,
        src_len: *mut u32,
    ) -> Option<usize> {
        match addr_resolution(src_addr as *const u16) {
            Addr::Ipv4(ip, port) => {
                info!("receive from ip {:x} port {}", ip, port);
                // 按 syscall 描述，这里需要把地址信息的长度写到用户给的 src_len 的位置
                unsafe {
                    *src_len = size_of::<IpAddr>() as u32;
                }
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
        //         if self.wr_type == SocketWrtype::RdOnly {
        //             self.wr_type = SocketWrtype::CLOSE;
        //         } else {
        //             self.wr_type = SocketWrtype::WrOnly;
        //         }
        //     },
        //     ShutdownFlag::SHUTWR => {
        //         if self.wr_type == SocketWrtype::WrOnly {
        //             self.wr_type = SocketWrtype::CLOSE;
        //         } else {
        //             self.wr_type = SocketWrtype::RdOnly;
        //         }
        //     },
        //     ShutdownFlag::SHUTRDWR => {
        //         self.wr_type = SocketWrtype::CLOSE;
        //     },
        // };
        0
    }
}
