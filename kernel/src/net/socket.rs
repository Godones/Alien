use alloc::boxed::Box;
use alloc::sync::Arc;

use rvfs::dentry::DirEntry;
use rvfs::file::{File, FileExtOps, FileMode, FileOps, OpenFlags};
use rvfs::mount::VfsMount;
use rvfs::superblock::{DataOps, Device};
use rvfs::StrResult;

use syscall_define::net::{Domain, SocketType, LOCAL_LOOPBACK_ADDR};
use syscall_define::LinuxErrno;

use crate::net::addr::IpV4Addr;
use crate::net::port::*;

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
    pub is_server: bool,
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
            is_server: false,
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
        file.access_inner().f_ops_ext = {
            let mut file_ext_ops = FileExtOps::empty();
            file_ext_ops.is_ready_read = socket_ready_to_read;
            file_ext_ops.is_ready_write = socket_ready_to_write;
            file_ext_ops
        };
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
        file.access_inner().f_ops_ext = {
            let mut file_ext_ops = FileExtOps::empty();
            file_ext_ops.is_ready_read = socket_ready_to_read;
            file_ext_ops.is_ready_write = socket_ready_to_write;
            file_ext_ops
        };
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
                    IpAddr::Ipv4(mut ip, mut port) => {
                        let old_port = self.local_addr.port().unwrap();
                        if port == 0 {
                            port = old_port;
                        } else if port != old_port {
                            delete_port2ip(old_port);
                        }
                        if ip == 0 {
                            // ip = 127.0.0,1
                            ip = LOCAL_LOOPBACK_ADDR
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

    pub fn accept(&self, buf: &mut [u8]) -> Option<(usize, IpAddr)> {
        assert!(self.is_server);
        let res = read_from_port_with_port_map(self.local_addr.port().unwrap(), buf).map(
            |(r, src_port)| {
                let ipaddr = find_ip_by_port(src_port).unwrap();
                (r, ipaddr)
            },
        );
        res
    }

    pub fn listening(&mut self, _bakc_log: usize) {
        self.listening = true;
        self.is_server = true;
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
                        create_connect_map(self.peer_addr.clone(), self.local_addr.clone());
                        write_to_port_with_port_map(self.local_addr.port().unwrap(), port, &be_ip);
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
                if self.is_server {
                    write_to_port_with_port_map(self.local_addr.port().unwrap(), port, message)
                } else {
                    write_to_port_with_s_c_map(self.local_addr.port().unwrap(), port, message)
                }
            }
            _ => None,
        }
    }

    pub fn recvfrom(&self, message: &mut [u8], _flags: usize) -> Option<(usize, IpAddr)> {
        let port = self.local_addr.port().unwrap();
        if self.is_server {
            read_from_port_with_s_c_map(port, message).map(|(r, src_port)| {
                let ipaddr = find_ip_by_port(src_port).unwrap();
                (r, ipaddr)
            })
        } else {
            let res = if self.peer_addr.is_valid() {
                read_from_port_with_port_map(port, message).map(|(r, src_port)| {
                    let ipaddr = find_ip_by_port(src_port).unwrap();
                    (r, ipaddr)
                })
            } else {
                read_from_port_with_s_c_map(port, message).map(|(r, src_port)| {
                    let ipaddr = find_ip_by_port(src_port).unwrap();
                    (r, ipaddr)
                })
            };
            res
        }
    }

    pub fn shutdown(&self, sdflag: ShutdownFlag) {
        match sdflag {
            ShutdownFlag::SHUTRD => {}
            ShutdownFlag::SHUTWR => {
                if self.is_server {
                    write_to_port_with_port_map(
                        self.local_addr.port().unwrap(),
                        self.peer_addr.port().unwrap(),
                        &[],
                    );
                } else {
                    write_to_port_with_s_c_map(
                        self.local_addr.port().unwrap(),
                        self.peer_addr.port().unwrap(),
                        &[],
                    );
                }
            }
            ShutdownFlag::SHUTRDWR => {}
        }
    }
}

fn socket_file_release(file: Arc<File>) -> StrResult<()> {
    error!("socket file release");
    let dentry_inner = file.f_dentry.access_inner();
    let inode_inner = dentry_inner.d_inode.access_inner();
    let data = inode_inner.data.as_ref().unwrap();
    let data = SocketData::from_ptr(data.data());
    let port = data.local_addr.port().unwrap();
    delete_port_with_port_map(port);
    if !data.is_server {
        // delete_port_with_s_c_map(port);
    } else {
        warn!("server socket release, we send a empty message to peer");
        write_to_port_with_port_map(port, data.peer_addr.port().unwrap(), &[]);
    }
    // delete_port2ip(port);
    Ok(())
}

fn socket_ready_to_read(file: Arc<File>) -> bool {
    let dentry_inner = file.f_dentry.access_inner();
    let inode_inner = dentry_inner.d_inode.access_inner();
    let data = inode_inner.data.as_ref().unwrap();
    let data = SocketData::from_ptr(data.data());
    let port = data.local_addr.port().unwrap();
    if data.is_server {
        if data.listening {
            check_port_have_data_with_port_map(port)
        } else {
            check_port_have_data_with_s_c_map(port)
        }
    } else {
        check_port_have_data_with_port_map(port)
    }
}

fn socket_ready_to_write(file: Arc<File>) -> bool {
    let dentry_inner = file.f_dentry.access_inner();
    let inode_inner = dentry_inner.d_inode.access_inner();
    let data = inode_inner.data.as_ref().unwrap();
    let data = SocketData::from_ptr(data.data());
    let port = data.peer_addr.port().unwrap();
    if data.is_server {
        if data.listening {
            panic!("server socket should not be ready to write");
        } else {
            check_port_can_write_with_port_map(port)
        }
    } else {
        check_port_can_write_with_s_c_map(port)
    }
}
