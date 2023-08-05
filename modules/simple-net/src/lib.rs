// #![no_std]
//
// extern crate alloc;
//
// use alloc::sync::Arc;
//
// use smoltcp::iface::{Interface, SocketSet};
// use smoltcp::phy::{Device, DeviceCapabilities, Medium, RxToken};
// use smoltcp::time::Instant;
// use smoltcp::wire::{EthernetAddress, IpAddress, IpCidr, Ipv4Address};
// use spin::Once;
// use virtio_drivers::device::net::VirtIONet;
// use virtio_drivers::Hal;
// use virtio_drivers::transport::Transport;
//
// use kernel_sync::Mutex;

pub mod common;
mod udp;
mod tcp;
mod unix;


//
// pub struct VirtIONetDeviceWrapper {
//     // inner: Arc<Mutex<VirtIONetDevice>>,
// }
//
// pub struct NetInterfaceWrapper {
//     ether_addr: Option<EthernetAddress>,
//     dev: Arc<VirtIONetDeviceWrapper>,
//     inner: Arc<Mutex<Interface>>,
//
// }
//
//
// pub static NET_DEVICE: Once<VirtIONetDeviceWrapper> = Once::new();
// pub static NET_INTERFACE: Once<NetInterfaceWrapper> = Once::new();
//
//
// impl VirtIONetDeviceWrapper {
//     pub fn new(dev: VirtIONetDevice) -> Self {
//         VirtIONetDeviceWrapper {
//             inner: Arc::new(Mutex::new(dev)),
//         }
//     }
//
//     pub fn mac_address(&self) -> EthernetAddress {
//         EthernetAddress(self.inner.lock().mac_address())
//     }
//
//     pub fn poll<F>(&mut self, f: F)
//         where
//             F: Fn(&[u8]),
//     {
//         let mut dev = self.inner.lock();
//         while dev.can_recv() {
//             match dev.receive() {
//                 Ok(buf) => {
//                     f(buf.packet());
//                 }
//                 Err(Error::NotReady) => break, // TODO: better method to avoid error type conversion
//                 Err(err) => {
//                     warn!("receive failed: {:?}", err);
//                     break;
//                 }
//             }
//         }
//     }
// }
//
// unsafe impl Sync for VirtIONetDeviceWrapper {}
//
// unsafe impl Send for VirtIONetDeviceWrapper {}
//
// impl Device for VirtIONetDeviceWrapper {
//     type RxToken<'a> = VirtioRxToken where Self: 'a;
//     type TxToken<'a> = VirtioTxToken where Self: 'a;
//
//     fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
//         match self.inner.lock().receive() {
//             Ok(buf) => Some((
//                 VirtioRxToken(self.inner.clone(), buf),
//                 VirtioTxToken(self.inner.clone()),
//             )),
//             Err(Error::NotReady) => None,
//             Err(err) => panic!("receive failed: {}", err),
//         }
//     }
//
//     fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
//         Some(VirtioTxToken(self.inner.clone()))
//     }
//
//     fn capabilities(&self) -> DeviceCapabilities {
//         let mut caps = DeviceCapabilities::default();
//         caps.max_transmission_unit = 1536;
//         caps.max_burst_size = Some(1);
//         caps.medium = Medium::Ethernet;
//         caps
//     }
// }
//
// pub struct VirtioRxToken(Arc<Mutex<VirtIONetDevice>>, RxBuffer);
//
// pub struct VirtioTxToken(Arc<Mutex<VirtIONetDevice>>);
//
// impl RxToken for VirtioRxToken {
//     fn consume<R, F>(self, f: F) -> R
//         where
//             F: FnOnce(&mut [u8]) -> R,
//     {
//         let mut rx_buf = self.1;
//         trace!(
//             "RECV {} bytes: {:02X?}",
//             rx_buf.packet_len(),
//             rx_buf.packet()
//         );
//         let result = f(rx_buf.packet_mut());
//         self.0.lock().recycle_rx_buffer(rx_buf).unwrap();
//         result
//     }
// }
//
// impl TxToken for VirtioTxToken {
//     fn consume<R, F>(self, len: usize, f: F) -> R
//         where
//             F: FnOnce(&mut [u8]) -> R,
//     {
//         let mut dev = self.0.lock();
//         let mut tx_buf = dev.new_tx_buffer(len);
//         let result = f(tx_buf.packet_mut());
//         trace!("SEND {} bytes: {:02X?}", len, tx_buf.packet());
//         dev.send(tx_buf).unwrap();
//         result
//     }
// }
//
//
// impl NetInterfaceWrapper {
//     pub fn new() -> Self {
//         let mut config = Config::new();
//         let dev = NET_DEVICE.get().expect("NETDEVICE not initialized");
//         let mut dev = VirtIONetDeviceWrapper {
//             inner: dev.inner.clone(),
//         };
//         config.random_seed = 0x2333;
//         if dev.capabilities().medium == Medium::Ethernet {
//             config.hardware_addr = Some(dev.mac_address().into());
//         }
//
//         let mut iface = Interface::new(config, &mut dev);
//         iface.update_ip_addrs(|ip_addrs| {
//             ip_addrs
//                 .push(IpCidr::new(IpAddress::from_str(IP).unwrap(), 24))
//                 .unwrap();
//         });
//
//         iface
//             .routes_mut()
//             .add_default_ipv4_route(Ipv4Address::from_str(GATEWAY).unwrap())
//             .unwrap();
//
//         NetInterfaceWrapper {
//             ether_addr: Some(dev.mac_address()),
//             inner: Arc::new(Mutex::new(iface)),
//             dev: Arc::new(dev),
//         }
//     }
//
//     pub fn ethernet_address(&self) -> Option<EthernetAddress> {
//         self.ether_addr
//     }
//
//
//     pub fn poll(&self, sockets: &Mutex<SocketSet>) {
//         let mut dev = self.dev.inner.lock();
//
//         // dev.poll(|buf| {
//         //     snoop_tcp_packet(buf).ok(); // preprocess TCP packets
//         // });
//
//         // let timestamp =
//         //     Instant::from_micros_const((current_time_nanos() / NANOS_PER_MICROS) as i64);
//         // let mut iface = self.iface.lock();
//         // let mut sockets = sockets.lock();
//         // iface.poll(timestamp, dev.deref_mut(), &mut sockets);
//     }
// }
//
//
// pub fn init_net<H: Hal, T: Transport, const QS: usize>(device: VirtIONet<H, T, QS>) {
//     unimplemented!()
// }
//
