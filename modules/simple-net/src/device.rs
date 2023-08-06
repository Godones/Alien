use alloc::sync::Arc;
use core::cell::RefCell;
use core::ops::{Deref, DerefMut};

use log::debug;
use preprint::pprintln;
use smoltcp::iface::SocketSet;
use smoltcp::phy::{Device, DeviceCapabilities, Medium, RxToken, TxToken};
use smoltcp::time::Instant;
use virtio_drivers::device::net::{RxBuffer, VirtIONet};
use virtio_drivers::transport::Transport;
use virtio_drivers::{Error, Hal};

use crate::common::STANDARD_MTU;
use crate::{KernelNetFunc, LISTENING_TABLE};

pub struct VirtIONetDeviceWrapper<H: Hal, T: Transport, const QS: usize> {
    inner: RefCell<VirtIONet<H, T, QS>>,
    timer: Arc<dyn KernelNetFunc>,
}

impl<H: Hal, T: Transport, const QS: usize> Deref for VirtIONetDeviceWrapper<H, T, QS> {
    type Target = RefCell<VirtIONet<H, T, QS>>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<H: Hal, T: Transport, const QS: usize> DerefMut for VirtIONetDeviceWrapper<H, T, QS> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<H: Hal, T: Transport, const QS: usize> VirtIONetDeviceWrapper<H, T, QS> {
    pub fn new(dev: VirtIONet<H, T, QS>, timer: Arc<dyn KernelNetFunc>) -> Self {
        VirtIONetDeviceWrapper {
            inner: RefCell::new(dev),
            timer,
        }
    }
}

unsafe impl<H: Hal, T: Transport, const QS: usize> Sync for VirtIONetDeviceWrapper<H, T, QS> {}

unsafe impl<H: Hal, T: Transport, const QS: usize> Send for VirtIONetDeviceWrapper<H, T, QS> {}

impl<H: Hal, T: Transport, const QS: usize> Device for VirtIONetDeviceWrapper<H, T, QS> {
    type RxToken<'a> = VirtioRxToken<'a, H, T, QS> where Self: 'a;
    type TxToken<'a> = VirtioTxToken<'a, H, T, QS> where Self: 'a;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        let mut dev = self.inner.borrow_mut();

        if !dev.can_recv() {
            return None;
        }

        match dev.receive() {
            Ok(buf) => Some((VirtioRxToken(&self.inner, buf), VirtioTxToken(&self.inner))),
            Err(Error::NotReady) => None,
            Err(err) => panic!("receive failed: {}", err),
        }
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        let dev = self.inner.borrow_mut();
        if !dev.can_send() {
            return None;
        }
        Some(VirtioTxToken(&self.inner))
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = 1514;
        caps.max_burst_size = None;
        caps.medium = Medium::Ethernet;
        caps
    }
}

pub struct VirtioRxToken<'a, H: Hal, T: Transport, const QS: usize>(
    &'a RefCell<VirtIONet<H, T, QS>>,
    RxBuffer,
);

pub struct VirtioTxToken<'a, H: Hal, T: Transport, const QS: usize>(
    &'a RefCell<VirtIONet<H, T, QS>>,
);

impl<H: Hal, T: Transport, const QS: usize> RxToken for VirtioRxToken<'_, H, T, QS> {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut rx_buf = self.1;
        debug!(
            "RECV {} bytes: {:02X?}",
            rx_buf.packet_len(),
            rx_buf.packet()
        );
        let result = f(rx_buf.packet_mut());
        self.0.borrow_mut().recycle_rx_buffer(rx_buf).unwrap();
        result
    }
    fn preprocess(&self, sockets: &mut SocketSet<'_>) {
        snoop_tcp_packet(self.1.packet(), sockets).ok();
    }
}

impl<H: Hal, T: Transport, const QS: usize> TxToken for VirtioTxToken<'_, H, T, QS> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut dev = self.0.borrow_mut();
        let mut tx_buf = dev.new_tx_buffer(len);
        let result = f(tx_buf.packet_mut());
        debug!("SEND {} bytes: {:02X?}", len, tx_buf.packet());
        dev.send(tx_buf).unwrap();
        result
    }
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
            // create a socket for the first incoming TCP packet, as the later accept() returns.
            LISTENING_TABLE.incoming_tcp_packet(src_addr, dst_addr, sockets);
        }
    }
    Ok(())
}

const GB: usize = 1000 * MB;
const MB: usize = 1000 * KB;
const KB: usize = 1000;

impl<H: Hal, T: Transport, const QS: usize> VirtIONetDeviceWrapper<H, T, QS> {
    pub fn bench_transmit_bandwidth(&mut self) {
        // 10 Gb
        const MAX_SEND_BYTES: usize = 10 * GB;
        let mut send_bytes: usize = 0;
        let mut past_send_bytes: usize = 0;
        let mut past_time: Instant = self.timer.now().into();

        // Send bytes
        while send_bytes < MAX_SEND_BYTES {
            if let Some(tx_token) = self.transmit(self.timer.now().into()) {
                VirtioTxToken::consume(tx_token, STANDARD_MTU, |tx_buf| {
                    tx_buf[0..12].fill(1);
                    // ether type: IPv4
                    tx_buf[12..14].copy_from_slice(&[0x08, 0x00]);
                    tx_buf[14..STANDARD_MTU].fill(1);
                });
                send_bytes += STANDARD_MTU;
            }

            let current_time: Instant = self.timer.now().into();
            if (current_time - past_time).secs() == 1 {
                let gb = ((send_bytes - past_send_bytes) * 8) / GB;
                let mb = (((send_bytes - past_send_bytes) * 8) % GB) / MB;
                let gib = (send_bytes - past_send_bytes) / GB;
                let mib = ((send_bytes - past_send_bytes) % GB) / MB;
                pprintln!(
                    "Transmit: {}.{:03}GBytes, Bandwidth: {}.{:03}Gbits/sec.",
                    gib,
                    mib,
                    gb,
                    mb
                );
                past_time = current_time;
                past_send_bytes = send_bytes;
            }
        }
    }

    pub fn bench_receive_bandwidth(&mut self) {
        // 10 Gb
        const MAX_RECEIVE_BYTES: usize = 10 * GB;
        let mut receive_bytes: usize = 0;
        let mut past_receive_bytes: usize = 0;
        let mut past_time: Instant = self.timer.now().into();
        // Receive bytes
        while receive_bytes < MAX_RECEIVE_BYTES {
            if let Some(rx_token) = self.receive(self.timer.now().into()) {
                VirtioRxToken::consume(rx_token.0, |rx_buf| {
                    receive_bytes += rx_buf.len();
                });
            }

            let current_time: Instant = self.timer.now().into();
            if (current_time - past_time).secs() == 1 {
                let gb = ((receive_bytes - past_receive_bytes) * 8) / GB;
                let mb = (((receive_bytes - past_receive_bytes) * 8) % GB) / MB;
                let gib = (receive_bytes - past_receive_bytes) / GB;
                let mib = ((receive_bytes - past_receive_bytes) % GB) / MB;
                pprintln!(
                    "Receive: {}.{:03}GBytes, Bandwidth: {}.{:03}Gbits/sec.",
                    gib,
                    mib,
                    gb,
                    mb
                );
                past_time = current_time;
                past_receive_bytes = receive_bytes;
            }
        }
    }
}
