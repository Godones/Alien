#![feature(new_uninit)]
#![feature(ip_in_core)]
#![allow(unused)]
#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;

use log::error;
use preprint::pprintln;
use smoltcp::iface::{SocketHandle, SocketSet};
use smoltcp::phy::Medium;
use smoltcp::time::Instant;
use smoltcp::wire::{EthernetAddress, IpAddress};
use spin::{Lazy, Mutex, Once};
use virtio_drivers::device::net::VirtIONet;
use virtio_drivers::transport::Transport;
use virtio_drivers::Hal;

use crate::common::{QEMU_GATEWAY, QEMU_IP};
use crate::device::VirtIONetDeviceWrapper;
use crate::interface::{
    NetInterface, NetInterfaceWrapper, NetInterfaceWrapperLoop, SocketSetWrapper,
};
use crate::listen_table::ListenTable;
use crate::loopdevice::LoopbackDev;

mod addr;
pub mod common;
mod device;
mod interface;
mod listen_table;
mod loopdevice;
pub mod tcp;
pub mod udp;
pub mod unix;

pub static NET_INTERFACE: Once<Box<dyn NetInterface>> = Once::new();
pub static SOCKET_SET: Lazy<SocketSetWrapper> = Lazy::new(|| SocketSetWrapper::new());
pub static LISTENING_TABLE: Lazy<ListenTable> = Lazy::new(|| ListenTable::new());
pub static KERNEL_NET_FUNC: Once<Arc<dyn KernelNetFunc>> = Once::new();

pub static UDP_PORT_REUSE: Lazy<Mutex<BTreeMap<u16, SocketHandle>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));

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
    fn yield_now(&self) -> bool; // equal to suspend in kernel
}

pub fn init_net<H: Hal + 'static, T: Transport + 'static, const QS: usize>(
    device: Option<VirtIONet<H, T, QS>>,
    kernel_func: Arc<dyn KernelNetFunc>,
    ip: Option<IpAddress>,
    gate_way: Option<IpAddress>,
    test: bool,
    loop_device: bool,
) {
    if !loop_device {
        let device = device.expect("no network device found");
        let mac_addr = EthernetAddress::from_bytes(device.mac_address().as_slice());
        let mut device = VirtIONetDeviceWrapper::new(device, kernel_func.clone());
        if test {
            device.bench_transmit_bandwidth();
        }
        let iface = NetInterfaceWrapper::new(device, kernel_func.clone());
        let ip = ip.unwrap_or(QEMU_IP.parse().expect("invalid IP address"));
        let gate_way =
            gate_way.unwrap_or(QEMU_GATEWAY.parse().expect("invalid gateway IP address"));
        iface.setup_ip_addr(ip, 24);
        iface.setup_gateway(gate_way);
        KERNEL_NET_FUNC.call_once(|| kernel_func);
        NET_INTERFACE.call_once(|| Box::new(iface));
        pprintln!("created net interface");
        pprintln!("  ether:    {}", mac_addr);
        pprintln!("  ip:       {}/{}", ip, 24);
        pprintln!("  gateway:  {}", gate_way);
    } else {
        let loop_device = LoopbackDev::new(Medium::Ip);
        let iface = NetInterfaceWrapperLoop::new(loop_device, kernel_func.clone());
        iface.setup_ip_addr(IpAddress::v4(127, 0, 0, 1), 8);
        KERNEL_NET_FUNC.call_once(|| kernel_func);
        NET_INTERFACE.call_once(|| Box::new(iface));
        pprintln!("created loop device");
        pprintln!("  ip:       {}/{}", IpAddress::v4(127, 0, 0, 1), 8);
    }
}

/// Poll the network stack.
///
/// It may receive packets from the NIC and process them, and transmit queued
/// packets to the NIC.
pub fn poll_interfaces() {
    SOCKET_SET.poll_interfaces();
}

fn snoop_tcp_packet(buf: &[u8], sockets: &mut SocketSet<'_>) -> Result<(), smoltcp::wire::Error> {
    use smoltcp::wire::{EthernetFrame, IpProtocol, Ipv4Packet, TcpPacket};

    let ether_frame = EthernetFrame::new_checked(buf)?;
    let ipv4_packet = Ipv4Packet::new_checked(ether_frame.payload())?;

    if ipv4_packet.next_header() == IpProtocol::Tcp {
        let tcp_packet = TcpPacket::new_checked(ipv4_packet.payload())?;
        let src_addr = (ipv4_packet.src_addr(), tcp_packet.src_port()).into();
        let dst_addr = (ipv4_packet.dst_addr(), tcp_packet.dst_port()).into();
        let is_first = tcp_packet.syn() && !tcp_packet.ack();
        if is_first {
            error!("TCP SYN packet: {} -> {}", src_addr, dst_addr);
            // create a socket for the first incoming TCP packet, as the later accept() returns.
            LISTENING_TABLE.incoming_tcp_packet(src_addr, dst_addr, sockets);
        }
    }
    Ok(())
}
