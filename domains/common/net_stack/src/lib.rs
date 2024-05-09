#![no_std]
#![forbid(unsafe_code)]

mod hexdump;
mod nic;
mod socket;

extern crate alloc;
use alloc::{boxed::Box, sync::Arc};
use core::{fmt::Debug, net::Ipv4Addr};

use basic::println;
use constants::{AlienError, AlienResult};
use interface::{Basic, DomainType, NetDeviceDomain, NetDomain};
use lose_net_stack::{connection::NetServer, MacAddress};
use rref::RRefVec;
use spin::Once;

use crate::nic::NetMod;

static NET_INTERFACE: Once<Arc<dyn NetDeviceDomain>> = Once::new();

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

impl NetStack {
    pub fn recv(&mut self, buf: &mut [u8]) -> usize {
        let len = loop {
            let res = NET_INTERFACE.get().unwrap().can_receive().unwrap();
            if res {
                let shared_buf = RRefVec::new(0, buf.len());
                let (shared_buf, len) = NET_INTERFACE.get().unwrap().receive(shared_buf).unwrap();
                buf.copy_from_slice(shared_buf.as_slice());
                break len;
            }
        };
        len
    }
}

impl Basic for NetStack {}

impl NetDomain for NetStack {
    fn init(&self, nic_domain_name: &str) -> AlienResult<()> {
        let nic_domain = basic::get_domain(nic_domain_name).expect("nic domain not found");
        match nic_domain {
            DomainType::NetDeviceDomain(nic_domain) => {
                NET_INTERFACE.call_once(|| nic_domain);
                println!("net stack init successed!");
                Ok(())
            }
            _ => Err(AlienError::EINVAL),
        }
    }
}

pub fn main() -> Box<dyn NetDomain> {
    Box::new(NetStack::new())
}
