use core::net::SocketAddrV4;

use downcast_rs::{impl_downcast, DowncastSync};
use gproxy::proxy;
use pconst::{
    io::PollEvents,
    net::{Domain, ShutdownFlag, SocketAddrIn, SocketType},
};
use rref::{RRef, RRefVec};

use super::AlienResult;
use crate::{Basic, DeviceBase};

pub type SocketID = usize;

#[proxy(NetDomainProxy)]
pub trait NetDomain: DeviceBase + Basic + DowncastSync {
    fn init(&self, nic_domain_name: &str) -> AlienResult<()>;
    fn socket(&self, domain: Domain, ty: SocketType, protocol: usize) -> AlienResult<SocketID>;
    fn socket_pair(&self, domain: Domain, ty: SocketType) -> AlienResult<(SocketID, SocketID)>;
    fn remove_socket(&self, socket_id: SocketID) -> AlienResult<()>;
    fn bind(&self, socket_id: SocketID, addr: &RRef<SocketAddrIn>)
        -> AlienResult<Option<SocketID>>;
    fn listen(&self, socket_id: SocketID, backlog: usize) -> AlienResult<()>;
    fn accept(&self, socket_id: SocketID) -> AlienResult<SocketID>;
    fn connect(&self, socket_id: SocketID, addr: &RRef<SocketAddrV4>) -> AlienResult<()>;

    fn recv_from(
        &self,
        socket_id: SocketID,
        arg_tuple: RRef<SocketArgTuple>,
    ) -> AlienResult<RRef<SocketArgTuple>>;
    fn sendto(
        &self,
        socket_id: SocketID,
        buf: &RRefVec<u8>,
        remote_addr: Option<&RRef<SocketAddrV4>>,
    ) -> AlienResult<usize>;
    fn shutdown(&self, socket_id: SocketID, how: ShutdownFlag) -> AlienResult<()>;

    fn remote_addr(
        &self,
        socket_id: SocketID,
        addr: RRef<SocketAddrIn>,
    ) -> AlienResult<RRef<SocketAddrIn>>;
    fn local_addr(
        &self,
        socket_id: SocketID,
        addr: RRef<SocketAddrIn>,
    ) -> AlienResult<RRef<SocketAddrIn>>;
    fn read_at(
        &self,
        socket_id: SocketID,
        offset: u64,
        buf: RRefVec<u8>,
    ) -> AlienResult<(RRefVec<u8>, usize)>;
    fn write_at(&self, socket_id: SocketID, offset: u64, buf: &RRefVec<u8>) -> AlienResult<usize>;
    fn poll(&self, socket_id: SocketID, events: PollEvents) -> AlienResult<PollEvents>;
}

pub struct SocketArgTuple {
    pub buf: RRefVec<u8>,
    pub addr: RRef<SocketAddrIn>,
    pub len: usize,
}

impl_downcast!(sync NetDomain);
