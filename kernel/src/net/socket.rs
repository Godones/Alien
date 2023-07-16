use alloc::boxed::Box;
use alloc::sync::Arc;

use rvfs::dentry::DirEntry;
use rvfs::file::{File, FileMode, FileOps, OpenFlags};
use rvfs::mount::VfsMount;
use rvfs::superblock::{DataOps, Device};
use rvfs::StrResult;

use syscall_define::net::{Domain, SocketType, LOCAL_LOOPBACK_ADDR};
use syscall_define::LinuxErrno;

use crate::net::addr::IpV4Addr;
use crate::net::port::{
    alloc_ephemeral_port, check_socket_file_exist, delete_port, delete_port2ip, find_ip_by_port,
    insert_port2ip, read_from_port, write_to_port,
};

use super::addr::IpAddr;
use super::ShutdownFlag;

#[derive(Debug, Clone)]
pub struct SocketData {
    /// socket 通信域  
    pub domain: Domain,
    /// 连接类型
    pub s_type: SocketType,
    /// 具体的通信协议
    pub protocol: usize,
    /// 连接的远端服务器的信息
    pub peer_addr: IpAddr,
    /// 本地的信息
    pub local_addr: IpAddr,

    pub listening: bool,
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
        // we need alloc a ephemeral
        let new_port = alloc_ephemeral_port();
        if new_port.is_none() {
            panic!("alloc ephemeral port failed");
        }
        let socket = Box::new(Self {
            domain,
            s_type,
            protocol,
            peer_addr: IpAddr::Empty,
            local_addr: IpAddr::Ipv4(0, new_port.unwrap()),
            listening: false,
        });

        let mut file_ops = FileOps::empty();
        file_ops.release = socket_file_release;
        let file = File::new(
            Arc::new(DirEntry::empty()),
            Arc::new(VfsMount::empty()),
            OpenFlags::O_RDWR,
            FileMode::FMODE_RDWR,
            file_ops,
        );
        // insert port2ip
        insert_port2ip(new_port.unwrap(), socket.local_addr.clone());
        file.f_dentry.access_inner().d_inode.access_inner().data = Some(socket);
        Arc::new(file)
    }
    pub fn new_with_data(data: SocketData) -> Arc<File> {
        let mut file_ops = FileOps::empty();
        file_ops.release = socket_file_release;
        let file = File::new(
            Arc::new(DirEntry::empty()),
            Arc::new(VfsMount::empty()),
            OpenFlags::O_RDWR,
            FileMode::FMODE_RDWR,
            file_ops,
        );
        file.f_dentry.access_inner().d_inode.access_inner().data = Some(Box::new(data));
        Arc::new(file)
    }

    pub fn socket_type(&self) -> SocketType {
        self.s_type
    }
    pub fn from_ptr(ptr: *const u8) -> &'static mut Self {
        unsafe { &mut *(ptr as *mut Self) }
    }

    pub fn to_be_ipv4_addr(&self) -> IpV4Addr {
        match self.local_addr {
            IpAddr::Ipv4(ip, port) => IpV4Addr {
                family: Domain::AF_INET as u16,
                port: port.to_be(),
                addr: ip.to_be(),
                zero: [0; 8],
            },
            _ => panic!("not ipv4 addr"),
        }
    }
    pub fn bind(&mut self, ip: IpAddr) -> Result<(), LinuxErrno> {
        match self.domain {
            Domain::AF_INET => {
                match ip {
                    IpAddr::Ipv4(ip, mut port) => {
                        let old_port = self.local_addr.port().unwrap();
                        if port == 0 {
                            port = old_port;
                        } else if port != old_port {
                            delete_port2ip(old_port);
                        }
                        insert_port2ip(port, IpAddr::Ipv4(ip, port));
                        // we only need reset ip
                        self.local_addr = IpAddr::Ipv4(ip, port);
                        Ok(())
                    }
                    _ => Err(LinuxErrno::EINVAL),
                }
            }
            Domain::AF_UNIX => match ip {
                IpAddr::LocalPath(path) => {
                    self.local_addr = IpAddr::LocalPath(path);
                    Ok(())
                }
                _ => Err(LinuxErrno::EINVAL),
            },
        }
    }

    pub fn listening(&mut self, _bakc_log: usize) {
        self.listening = true;
    }

    pub fn connect(&mut self, ip: IpAddr) -> Result<(), LinuxErrno> {
        match self.domain {
            Domain::AF_INET => {
                match ip {
                    IpAddr::Ipv4(ip, port) => {
                        self.peer_addr = IpAddr::Ipv4(ip, port);
                        // send self ip to peer
                        let be_ip = self.local_addr.to_be_ipv4().unwrap();
                        let be_ip = be_ip.to_bytes();
                        write_to_port(self.local_addr.port().unwrap(), port, &be_ip);
                        warn!("[connect] send self ip to peer, first hand");
                        Ok(())
                    }
                    _ => Err(LinuxErrno::EINVAL),
                }
            }
            Domain::AF_UNIX => match ip {
                IpAddr::LocalPath(path) => {
                    if check_socket_file_exist(&path) {
                        self.peer_addr = IpAddr::LocalPath(path);
                        Ok(())
                    } else {
                        Err(LinuxErrno::ENOENT)
                    }
                }
                _ => Err(LinuxErrno::EINVAL),
            },
        }
    }
    pub fn send_to(
        &self,
        message: &[u8],
        _flags: usize,
        dest_addr: Option<IpAddr>,
    ) -> Option<usize> {
        let dest_addr = if dest_addr.is_none() {
            self.peer_addr.clone()
        } else {
            dest_addr.unwrap()
        };
        match dest_addr {
            IpAddr::Ipv4(ip, port) => {
                assert!(ip == 0 || ip == LOCAL_LOOPBACK_ADDR);
                write_to_port(self.local_addr.port().unwrap(), port, message)
            }
            _ => None,
        }
    }

    pub fn recvfrom(&self, message: &mut [u8], _flags: usize) -> Option<(usize, IpAddr)> {
        let port = self.local_addr.port().unwrap();
        let res = read_from_port(port, message).map(|(r, src_port)| {
            let ipaddr = find_ip_by_port(src_port).unwrap();
            (r, ipaddr)
        });
        res
    }

    pub fn shutdown(&self, _sdflag: ShutdownFlag) -> isize {
        0
    }
}

fn socket_file_release(file: Arc<File>) -> StrResult<()> {
    error!("socket file release");
    let dentry_inner = file.f_dentry.access_inner();
    let inode_inner = dentry_inner.d_inode.access_inner();
    let data = inode_inner.data.as_ref().unwrap();
    let data = SocketData::from_ptr(data.data());
    let port = data.local_addr.port().unwrap();
    delete_port2ip(port);
    delete_port(port);
    Ok(())
}
