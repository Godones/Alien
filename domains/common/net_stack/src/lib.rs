#![no_std]
#![forbid(unsafe_code)]

mod error;
mod hexdump;
mod nic;
mod socket;
mod socket_pair;

extern crate alloc;
use alloc::{boxed::Box, collections::BTreeMap, sync::Arc};
use core::{
    cmp,
    fmt::Debug,
    net::{Ipv4Addr, SocketAddrV4},
    ops::Deref,
    sync::atomic::AtomicUsize,
};

use basic::{
    constants::{
        io::PollEvents,
        net::{Domain, ShutdownFlag, SocketAddrIn, SocketType},
    },
    println,
    sync::Mutex,
    AlienError, AlienResult,
};
use downcast_rs::{impl_downcast, DowncastSync};
use interface::{
    Basic, DeviceBase, DomainType, NetDeviceDomain, NetDomain, SocketArgTuple, SocketID,
};
use log::{debug, info};
use lose_net_stack::{connection::NetServer, MacAddress};
use rref::{RRef, RRefVec};
use spin::Once;

use crate::{error::to_alien_error, nic::NetMod, socket::Socket, socket_pair::SocketPair};

static NET_INTERFACE: Once<Arc<dyn NetDeviceDomain>> = Once::new();
static SOCKET_MAP: Mutex<BTreeMap<SocketID, Arc<dyn SocketFile>>> = Mutex::new(BTreeMap::new());
static SOCKET_ID: AtomicUsize = AtomicUsize::new(0);

pub trait SocketFile: Send + Sync + DowncastSync {
    fn write_at(&self, _offset: usize, buffer: &[u8]) -> AlienResult<usize>;
    fn read_at(&self, _offset: usize, buffer: &mut [u8]) -> AlienResult<usize>;
    fn poll(&self, events: PollEvents) -> AlienResult<PollEvents>;
}

impl_downcast!(sync SocketFile);

pub struct NetStack {
    net_server: Arc<NetServer<NetMod>>,
}

impl Debug for NetStack {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "NetStack")
    }
}

impl NetStack {
    pub fn new() -> Self {
        let net_server = Arc::new(NetServer::<NetMod>::new(
            MacAddress::new([0x52, 0x54, 0x00, 0x12, 0x34, 0x56]),
            Ipv4Addr::new(10, 0, 2, 15),
        ));
        Self { net_server }
    }
}

impl Basic for NetStack {}

impl DeviceBase for NetStack {
    fn handle_irq(&self) -> AlienResult<()> {
        info!("<handle_irq> NetStack handle_irq");
        let nic = NET_INTERFACE.get().unwrap();
        nic.handle_irq()?;
        let mut shared_buf = RRefVec::new(0, 1600);
        while nic.can_receive()? {
            let (mut buf, len) = nic.receive(shared_buf).unwrap();
            debug!("recv data {} bytes", len);
            self.net_server
                .analysis_net_data(&mut buf.as_mut_slice()[..len]);
            shared_buf = buf;
        }
        info!("<handle_irq> net stack handle irq success");
        Ok(())
    }
}

impl NetDomain for NetStack {
    fn init(&self, nic_domain_name: &str) -> AlienResult<()> {
        let nic_domain = basic::get_domain(nic_domain_name).expect("nic domain not found");
        match nic_domain {
            DomainType::NetDeviceDomain(nic_domain) => {
                NET_INTERFACE.call_once(|| nic_domain);
                println!("net stack init successes!");
                Ok(())
            }
            _ => Err(AlienError::EINVAL),
        }
    }

    fn socket(&self, domain: Domain, ty: SocketType, _protocol: usize) -> AlienResult<SocketID> {
        let socket = Socket::new(&self.net_server, domain, ty);
        let id = add_socket_file(socket);
        Ok(id)
    }

    fn socket_pair(&self, _domain: Domain, _ty: SocketType) -> AlienResult<(SocketID, SocketID)> {
        let socket_pair = SocketPair::new();
        let id = add_socket_file(socket_pair);
        // same socket pair
        Ok((id, id))
    }

    fn remove_socket(&self, socket_id: SocketID) -> AlienResult<()> {
        remove_socket_file(socket_id);
        Ok(())
    }

    fn bind(
        &self,
        socket_id: SocketID,
        socket_addr: &RRef<SocketAddrIn>,
    ) -> AlienResult<Option<SocketID>> {
        let socket = SOCKET_MAP
            .lock()
            .get(&socket_id)
            .ok_or(AlienError::EINVAL)?
            .clone()
            .downcast_arc::<Socket>()
            .map_err(|_| AlienError::EINVAL)?;
        let port = socket_addr.in_port.to_be();
        match socket.net_type {
            SocketType::SOCK_STREAM => {
                if self.net_server.tcp_is_used(port) {
                    let sock = socket.reuse(&self.net_server, port);
                    let new_socket_id = add_socket_file(Arc::new(sock));
                    return Ok(Some(new_socket_id));
                }
            }
            SocketType::SOCK_DGRAM => {
                if self.net_server.udp_is_used(port) {
                    let sock = socket.reuse(&self.net_server, port);
                    let new_socket_id = add_socket_file(Arc::new(sock));
                    return Ok(Some(new_socket_id));
                }
            }
            SocketType::SOCK_RAW => {}
            ty => panic!("can't bind this type of socket: {:?}", ty),
        }
        let local = SocketAddrV4::new(socket_addr.addr, port);
        socket
            .inner
            .clone()
            .bind(local)
            .map_err(|_| AlienError::EALREADY)?;
        info!("socket_addr: {:?}", *socket_addr.deref());
        Ok(None)
    }

    fn listen(&self, socket_id: SocketID, _backlog: usize) -> AlienResult<()> {
        let socket = SOCKET_MAP
            .lock()
            .get(&socket_id)
            .ok_or(AlienError::EINVAL)?
            .clone()
            .downcast_arc::<Socket>()
            .map_err(|_| AlienError::EINVAL)?;
        socket.inner.clone().listen().expect("listen failed");
        Ok(())
    }

    fn accept(&self, socket_id: SocketID) -> AlienResult<SocketID> {
        let socket = SOCKET_MAP
            .lock()
            .get(&socket_id)
            .ok_or(AlienError::EINVAL)?
            .clone()
            .downcast_arc::<Socket>()
            .map_err(|_| AlienError::EINVAL)?;
        let new_socket = socket.inner.accept();
        match new_socket {
            Ok(new_socket) => {
                let new_socket = Socket::new_with_inner(socket.domain, socket.net_type, new_socket);
                let new_socket_id = add_socket_file(new_socket);
                Ok(new_socket_id)
            }
            Err(e) => Err(to_alien_error(e)),
        }
    }

    fn connect(&self, socket_id: SocketID, addr: &RRef<SocketAddrV4>) -> AlienResult<()> {
        let socket = SOCKET_MAP
            .lock()
            .get(&socket_id)
            .ok_or(AlienError::EINVAL)?
            .clone()
            .downcast_arc::<Socket>()
            .map_err(|_| AlienError::EINVAL)?;
        socket
            .inner
            .clone()
            .connect(*addr.deref())
            .map_err(to_alien_error)
    }

    // todo!(for reuse arg, if there is no data, we set socket_arg_tuple.len ==0 and return ok(it));
    fn recv_from(
        &self,
        socket_id: SocketID,
        mut socket_arg_tuple: RRef<SocketArgTuple>,
    ) -> AlienResult<RRef<SocketArgTuple>> {
        let socket = SOCKET_MAP
            .lock()
            .get(&socket_id)
            .ok_or(AlienError::EINVAL)?
            .clone()
            .downcast_arc::<Socket>()
            .map_err(|_| AlienError::EINVAL)?;
        let res = socket.recv_from();

        match res {
            Err(AlienError::EBLOCKING) => {
                socket_arg_tuple.len = 0;
                Ok(socket_arg_tuple)
            }
            Ok((data, remote_addr)) => {
                let socket_add_in = SocketAddrIn {
                    family: Domain::AF_INET as u16,
                    in_port: remote_addr.port().to_be(),
                    addr: *remote_addr.ip(),
                    sin_zero: [0; 8],
                };
                let rlen = cmp::min(data.len(), socket_arg_tuple.buf.len());
                socket_arg_tuple.buf.as_mut_slice()[..rlen].copy_from_slice(&data[..rlen]);
                *socket_arg_tuple.addr = socket_add_in;
                socket_arg_tuple.len = rlen;
                Ok(socket_arg_tuple)
            }
            Err(e) => Err(e),
        }
    }

    fn sendto(
        &self,
        socket_id: SocketID,
        buf: &RRefVec<u8>,
        remote_addr: Option<&RRef<SocketAddrV4>>,
    ) -> AlienResult<usize> {
        let socket = SOCKET_MAP
            .lock()
            .get(&socket_id)
            .ok_or(AlienError::EINVAL)?
            .clone()
            .downcast_arc::<Socket>()
            .map_err(|_| AlienError::EINVAL)?;

        if socket.inner.get_local().unwrap().port() == 0 {
            socket
                .inner
                .clone()
                .bind(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 0))
                .map_err(|_| AlienError::EALREADY)?;
        }

        let addr = remote_addr.map(|addr| *addr.deref());

        let rlen = socket.inner.clone().sendto(buf.as_slice(), addr).unwrap();
        Ok(rlen)
    }

    fn shutdown(&self, socket_id: SocketID, _how: ShutdownFlag) -> AlienResult<()> {
        let socket = SOCKET_MAP
            .lock()
            .get(&socket_id)
            .ok_or(AlienError::EINVAL)?
            .clone()
            .downcast_arc::<Socket>()
            .map_err(|_| AlienError::EINVAL)?;
        socket.inner.clone().close().map_err(to_alien_error)
    }

    fn remote_addr(
        &self,
        socket_id: SocketID,
        mut addr: RRef<SocketAddrIn>,
    ) -> AlienResult<RRef<SocketAddrIn>> {
        let socket = SOCKET_MAP
            .lock()
            .get(&socket_id)
            .ok_or(AlienError::EINVAL)?
            .clone()
            .downcast_arc::<Socket>()
            .map_err(|_| AlienError::EINVAL)?;
        let remote = socket.inner.get_remote().unwrap();
        let socket_add_in = SocketAddrIn {
            family: Domain::AF_INET as u16,
            in_port: remote.port().to_be(),
            addr: *remote.ip(),
            sin_zero: [0; 8],
        };
        *addr = socket_add_in;
        Ok(addr)
    }

    fn local_addr(
        &self,
        socket_id: SocketID,
        mut addr: RRef<SocketAddrIn>,
    ) -> AlienResult<RRef<SocketAddrIn>> {
        let socket = SOCKET_MAP
            .lock()
            .get(&socket_id)
            .ok_or(AlienError::EINVAL)?
            .clone()
            .downcast_arc::<Socket>()
            .map_err(|_| AlienError::EINVAL)?;
        let local = socket.inner.get_local().unwrap();
        let socket_add_in = SocketAddrIn {
            family: Domain::AF_INET as u16,
            in_port: local.port().to_be(),
            addr: *local.ip(),
            sin_zero: [0; 8],
        };
        *addr = socket_add_in;
        Ok(addr)
    }

    fn read_at(
        &self,
        socket_id: SocketID,
        offset: u64,
        mut buf: RRefVec<u8>,
    ) -> AlienResult<(RRefVec<u8>, usize)> {
        let socket = SOCKET_MAP
            .lock()
            .get(&socket_id)
            .ok_or(AlienError::EINVAL)?
            .clone();
        let r = socket.read_at(offset as usize, buf.as_mut_slice())?;
        Ok((buf, r))
    }

    fn write_at(&self, socket_id: SocketID, offset: u64, buf: &RRefVec<u8>) -> AlienResult<usize> {
        let socket = SOCKET_MAP
            .lock()
            .get(&socket_id)
            .ok_or(AlienError::EINVAL)?
            .clone();
        socket.write_at(offset as usize, buf.as_slice())
    }

    fn poll(&self, socket_id: SocketID, events: PollEvents) -> AlienResult<PollEvents> {
        let socket = SOCKET_MAP
            .lock()
            .get(&socket_id)
            .ok_or(AlienError::EINVAL)?
            .clone();
        socket.poll(events)
    }
}

fn add_socket_file(socket: Arc<dyn SocketFile>) -> SocketID {
    let id = SOCKET_ID.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
    SOCKET_MAP.lock().insert(id, socket);
    id
}

fn remove_socket_file(socket_id: SocketID) {
    SOCKET_MAP.lock().remove(&socket_id);
}

pub fn main() -> Box<dyn NetDomain> {
    Box::new(NetStack::new())
}
