use alloc::sync::Arc;
use kernel_sync::Mutex;
use spin::Once;
use core::str::FromStr;

use virtio_drivers::device::net::{ VirtIONet, RxBuffer,};
use virtio_drivers::transport::mmio::MmioTransport;
use virtio_drivers::Error;

use crate::driver::hal::HalImpl;
use lazy_static::lazy_static;

use smoltcp::phy::{Device, DeviceCapabilities, Medium, RxToken, TxToken, };
use smoltcp::wire::{EthernetAddress, IpAddress, IpCidr, Ipv4Address, };
use smoltcp::time::Instant;
use smoltcp::iface::{Config,Interface};


pub const NET_BUFFER_LEN: usize = 2048;
pub const NET_QUEUE_SIZE: usize = 16;

const IP: &str = "10.0.2.15"; // QEMU user networking default IP
const GATEWAY: &str = "10.0.2.2"; // QEMU user networking gateway

type VirtIONetDevice = VirtIONet<HalImpl, MmioTransport, NET_QUEUE_SIZE>;


pub struct VirtIONetWrapper {
    inner: Arc<Mutex<VirtIONetDevice>>,
}

pub struct NetInterfaceWrapper {
    inner: Arc<Mutex<Interface>>,
}

lazy_static!(
    pub static ref NET_DEVICE: Once<VirtIONetWrapper> = Once::new();
    pub static ref NET_INTERFACE: Once<NetInterfaceWrapper> = Once::new();
);



impl VirtIONetWrapper {
    pub fn new(dev: VirtIONetDevice) -> Self {
        VirtIONetWrapper {
            inner: Arc::new(Mutex::new(dev)),
        }
    }

    pub fn mac_address(&self) -> EthernetAddress {
        EthernetAddress(self.inner.lock().mac_address())
    }

}

unsafe impl Sync for VirtIONetWrapper {}

unsafe impl Send for VirtIONetWrapper {}

impl Device for VirtIONetWrapper {
    type RxToken<'a> = VirtioRxToken where Self: 'a;
    type TxToken<'a> = VirtioTxToken where Self: 'a;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        match self.inner.lock().receive() {
            Ok(buf) => Some((
                VirtioRxToken(self.inner.clone(), buf),
                VirtioTxToken(self.inner.clone()),
            )),
            Err(Error::NotReady) => None,
            Err(err) => panic!("receive failed: {}", err),
        }
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        Some(VirtioTxToken(self.inner.clone()))
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = 1536;
        caps.max_burst_size = Some(1);
        caps.medium = Medium::Ethernet;
        caps
    }
}

pub struct VirtioRxToken(Arc<Mutex<VirtIONetDevice>>, RxBuffer);
pub struct VirtioTxToken(Arc<Mutex<VirtIONetDevice>>);

impl RxToken for VirtioRxToken {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut rx_buf = self.1;
        trace!(
            "RECV {} bytes: {:02X?}",
            rx_buf.packet_len(),
            rx_buf.packet()
        );
        let result = f(rx_buf.packet_mut());
        self.0.lock().recycle_rx_buffer(rx_buf).unwrap();
        result
    }
}

impl TxToken for VirtioTxToken {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut dev = self.0.lock();
        let mut tx_buf = dev.new_tx_buffer(len);
        let result = f(tx_buf.packet_mut());
        trace!("SEND {} bytes: {:02X?}", len, tx_buf.packet());
        dev.send(tx_buf).unwrap();
        result
    }
}


impl NetInterfaceWrapper {
    pub fn new() -> Self {
        let mut config = Config::new();
        let dev = NET_DEVICE.get().expect("NETDEVICE not initialized");
        let mut dev = VirtIONetWrapper{
            inner: dev.inner.clone(),
        };
        config.random_seed = 0x2333;
        if dev.capabilities().medium == Medium::Ethernet {
            config.hardware_addr = Some(dev.mac_address().into());
        }
    
        let mut iface = Interface::new(config, &mut dev);
        iface.update_ip_addrs(|ip_addrs| {
            ip_addrs
                .push(IpCidr::new(IpAddress::from_str(IP).unwrap(), 24))
                .unwrap();
        });
    
        iface
            .routes_mut()
            .add_default_ipv4_route(Ipv4Address::from_str(GATEWAY).unwrap())
            .unwrap();

        NetInterfaceWrapper {
            inner: Arc::new(Mutex::new(iface))
        }
    }


    // pub fn poll(&self, sockets: &Mutex<SocketSet>) {
    //     let mut config = Config::new();
    //     let dev = NET_DEVICE.get().expect("NETDEVICE not initialized");
    //     let mut dev = VirtIONetWrapper{
    //         inner: dev.inner.clone(),
    //     };
    //     dev.poll(|buf| {
    //         snoop_tcp_packet(buf).ok(); // preprocess TCP packets
    //     });

    //     let timestamp =
    //         Instant::from_micros_const((current_time_nanos() / NANOS_PER_MICROS) as i64);
    //     let mut iface = self.iface.lock();
    //     let mut sockets = sockets.lock();
    //     iface.poll(timestamp, dev.deref_mut(), &mut sockets);
    // }
}

// fn snoop_tcp_packet(buf: &[u8]) -> Result<(), smoltcp::wire::Error> {
//     use smoltcp::wire::{EthernetFrame, IpProtocol, Ipv4Packet, TcpPacket};

//     let ether_frame = EthernetFrame::new_checked(buf)?;
//     let ipv4_packet = Ipv4Packet::new_checked(ether_frame.payload())?;

//     if ipv4_packet.next_header() == IpProtocol::Tcp {
//         let tcp_packet = TcpPacket::new_checked(ipv4_packet.payload())?;
//         let src_addr = SocketAddr::new(ipv4_packet.src_addr().into(), tcp_packet.src_port());
//         let dst_addr = SocketAddr::new(ipv4_packet.dst_addr().into(), tcp_packet.dst_port());
//         let is_first = tcp_packet.syn() && !tcp_packet.ack();
//         if is_first {
//             // create a socket for the first incoming TCP packet, as the later accept() returns.
//             LISTEN_TABLE.incoming_tcp_packet(src_addr, dst_addr);
//         }
//     }
//     Ok(())
// }
