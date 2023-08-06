use alloc::sync::Arc;
use alloc::vec;
use core::ops::DerefMut;

use log::debug;
use smoltcp::iface::{Config, Interface, SocketHandle, SocketSet};
use smoltcp::socket;
use smoltcp::socket::AnySocket;
use smoltcp::wire::{EthernetAddress, HardwareAddress, IpAddress, IpCidr};
use virtio_drivers::transport::Transport;
use virtio_drivers::Hal;

use kernel_sync::Mutex;

use crate::common::{TCP_RX_BUF_LEN, TCP_TX_BUF_LEN, UDP_RX_BUF_LEN, UDP_TX_BUF_LEN};
use crate::device::VirtIONetDeviceWrapper;
use crate::{KernelNetFunc, NET_INTERFACE};

pub trait NetInterface: Send + Sync {
    fn ethernet_address(&self) -> EthernetAddress;
    fn setup_ip_addr(&self, ip: IpAddress, prefix_len: u8);
    fn setup_gateway(&self, gateway: IpAddress);
    fn poll(&self, sockets: &Mutex<SocketSet>);
    fn raw_interface(&self) -> &Mutex<Interface>;
}

pub struct NetInterfaceWrapper<H: Hal, T: Transport, const QS: usize> {
    dev: Mutex<VirtIONetDeviceWrapper<H, T, QS>>,
    interface: Mutex<Interface>,
    timer: Arc<dyn KernelNetFunc>,
}

impl<H: Hal, T: Transport, const QS: usize> NetInterfaceWrapper<H, T, QS> {
    pub fn new(dev: VirtIONetDeviceWrapper<H, T, QS>, timer: Arc<dyn KernelNetFunc>) -> Self {
        let ether_addr = dev.borrow().mac_address();
        let ether_addr = EthernetAddress::from_bytes(&ether_addr);
        let mut config = Config::new(HardwareAddress::Ethernet(ether_addr));
        config.random_seed = 0x9898998;
        let mut dev = dev;
        let time = timer.now().into();
        let interface = Interface::new(config, &mut dev, time);
        Self {
            dev: Mutex::new(dev),
            interface: Mutex::new(interface),
            timer,
        }
    }
}

impl<H: Hal, T: Transport, const QS: usize> NetInterface for NetInterfaceWrapper<H, T, QS> {
    fn ethernet_address(&self) -> EthernetAddress {
        let ether_addr = self.dev.lock().borrow().mac_address();
        EthernetAddress::from_bytes(ether_addr.as_slice())
    }

    fn setup_ip_addr(&self, ip: IpAddress, prefix_len: u8) {
        let mut interface = self.interface.lock();
        interface.update_ip_addrs(|ips| {
            ips.push(IpCidr::new(ip, prefix_len)).unwrap();
        })
    }

    fn setup_gateway(&self, gateway: IpAddress) {
        let mut interface = self.interface.lock();
        match gateway {
            IpAddress::Ipv4(v4) => interface.routes_mut().add_default_ipv4_route(v4).unwrap(),
            _ => None,
        };
    }

    fn poll(&self, sockets: &Mutex<SocketSet>) {
        let mut dev = self.dev.lock();
        let mut interface = self.interface.lock();
        let mut sockets = sockets.lock();
        let timestamp = self.timer.now();
        interface.poll(timestamp.into(), dev.deref_mut(), &mut sockets);
    }

    fn raw_interface(&self) -> &Mutex<Interface> {
        &self.interface
    }
}

pub struct SocketSetWrapper<'a>(Mutex<SocketSet<'a>>);

impl<'a> SocketSetWrapper<'a> {
    pub fn new() -> Self {
        Self(Mutex::new(SocketSet::new(vec![])))
    }

    pub fn new_tcp_socket() -> socket::tcp::Socket<'a> {
        let tcp_rx_buffer = socket::tcp::SocketBuffer::new(vec![0; TCP_RX_BUF_LEN]);
        let tcp_tx_buffer = socket::tcp::SocketBuffer::new(vec![0; TCP_TX_BUF_LEN]);
        socket::tcp::Socket::new(tcp_rx_buffer, tcp_tx_buffer)
    }

    pub fn new_udp_socket() -> socket::udp::Socket<'a> {
        let udp_rx_buffer = socket::udp::PacketBuffer::new(
            vec![socket::udp::PacketMetadata::EMPTY; 8],
            vec![0; UDP_RX_BUF_LEN],
        );
        let udp_tx_buffer = socket::udp::PacketBuffer::new(
            vec![socket::udp::PacketMetadata::EMPTY; 8],
            vec![0; UDP_TX_BUF_LEN],
        );
        socket::udp::Socket::new(udp_rx_buffer, udp_tx_buffer)
    }

    pub fn add<T: AnySocket<'a>>(&self, socket: T) -> SocketHandle {
        let handle = self.0.lock().add(socket);
        debug!("socket {}: created", handle);
        handle
    }

    pub fn with_socket<T: AnySocket<'a>, R, F>(&self, handle: SocketHandle, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        let set = self.0.lock();
        let socket = set.get(handle);
        f(socket)
    }

    pub fn with_socket_mut<T: AnySocket<'a>, R, F>(&self, handle: SocketHandle, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut set = self.0.lock();
        let socket = set.get_mut(handle);
        f(socket)
    }

    /// The NET_INTERFACE should be initialized before calling this function.
    pub fn poll_interfaces(&self) {
        NET_INTERFACE.get().unwrap().poll(&self.0);
    }

    pub fn remove(&self, handle: SocketHandle) {
        self.0.lock().remove(handle);
        debug!("socket {}: destroyed", handle);
    }
}
