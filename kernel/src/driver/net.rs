use alloc::sync::Arc;
use kernel_sync::Mutex;
use spin::Once;

use virtio_drivers::device::net::{ VirtIONet, RxBuffer,};
use virtio_drivers::transport::mmio::MmioTransport;
use virtio_drivers::Error;

use crate::driver::hal::HalImpl;
use lazy_static::lazy_static;

use smoltcp::phy::{Device, DeviceCapabilities, Medium, RxToken, TxToken, };
use smoltcp::wire::EthernetAddress;
use smoltcp::time::Instant;


pub const NET_BUFFER_LEN: usize = 2048;
pub const NET_QUEUE_SIZE: usize = 16;

type VirtIONetDevice = VirtIONet<HalImpl, MmioTransport, NET_QUEUE_SIZE>;


pub struct VirtIONetWrapper {
    inner: Arc<Mutex<VirtIONetDevice>>,
}

lazy_static!(
    pub static ref NET_DEVICE: Once<VirtIONetWrapper> = Once::new();
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
