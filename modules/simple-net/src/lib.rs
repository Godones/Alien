#![feature(new_uninit)]
#![feature(ip_in_core)]
#![allow(unused)]
#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use alloc::sync::Arc;

use preprint::pprintln;
use smoltcp::time::Instant;
use smoltcp::wire::{EthernetAddress, IpAddress};
use spin::{Lazy, Once};
use virtio_drivers::device::net::VirtIONet;
use virtio_drivers::transport::Transport;
use virtio_drivers::Hal;

use crate::common::{QEMU_GATEWAY, QEMU_IP};
use crate::device::VirtIONetDeviceWrapper;
use crate::interface::{NetInterface, NetInterfaceWrapper, SocketSetWrapper};
use crate::listen_table::ListenTable;

mod addr;
pub mod common;
mod device;
mod interface;
mod listen_table;
pub mod tcp;
pub mod udp;
pub mod unix;

pub static NET_INTERFACE: Once<Box<dyn NetInterface>> = Once::new();
pub static SOCKET_SET: Lazy<SocketSetWrapper> = Lazy::new(|| SocketSetWrapper::new());
pub static LISTENING_TABLE: Lazy<ListenTable> = Lazy::new(|| ListenTable::new());
pub static KERNEL_NET_FUNC: Once<Arc<dyn KernelNetFunc>> = Once::new();

pub struct NetInstant {
    pub micros: i64,
}

impl Into<Instant> for NetInstant {
    fn into(self) -> Instant {
        Instant::from_micros(self.micros)
    }
}

pub trait KernelNetFunc: Send + Sync {
    fn now(&self) -> NetInstant;
    fn yield_now(&self); // equal to suspend in kernel
}

pub fn init_net<H: Hal + 'static, T: Transport + 'static, const QS: usize>(
    device: VirtIONet<H, T, QS>,
    kernel_func: Arc<dyn KernelNetFunc>,
    ip: Option<IpAddress>,
    gate_way: Option<IpAddress>,
    test: bool,
) {
    let mac_addr = EthernetAddress::from_bytes(device.mac_address().as_slice());
    let mut device = VirtIONetDeviceWrapper::new(device, kernel_func.clone());
    if test {
        device.bench_transmit_bandwidth();
    }
    let iface = NetInterfaceWrapper::new(device, kernel_func.clone());
    let ip = ip.unwrap_or(QEMU_IP.parse().expect("invalid IP address"));
    let gate_way = gate_way.unwrap_or(QEMU_GATEWAY.parse().expect("invalid gateway IP address"));
    iface.setup_ip_addr(ip, 24);
    iface.setup_gateway(gate_way);
    KERNEL_NET_FUNC.call_once(|| kernel_func);
    NET_INTERFACE.call_once(|| Box::new(iface));
    pprintln!("created net interface");
    pprintln!("  ether:    {}", mac_addr);
    pprintln!("  ip:       {}/{}", ip, 24);
    pprintln!("  gateway:  {}", gate_way);
}
