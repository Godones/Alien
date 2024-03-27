#![no_std]

mod virtio_net;

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use constants::AlienResult;
use core::fmt::{Debug, Formatter, Result};
use interface::{Basic, DeviceBase, DeviceInfo, NetDomain, RxBufferWrapper, TxBufferWrapper};
use ksync::Mutex;
use rref::RRefVec;
use spin::Once;
use virtio_drivers::device::net::{RxBuffer, TxBuffer};

use virtio_net::{VirtIoNetWrapper, NET_QUEUE_SIZE};

pub struct VirtIoNetDomain {
    tx_buf_map: Mutex<BTreeMap<usize, TxBuffer>>,
    rx_buf_map: Mutex<BTreeMap<usize, RxBuffer>>,
}

impl VirtIoNetDomain {
    pub fn new() -> Self {
        Self {
            tx_buf_map: Mutex::new(BTreeMap::new()),
            rx_buf_map: Mutex::new(BTreeMap::new()),
        }
    }
}

impl Debug for VirtIoNetDomain {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        writeln!(f, "Net Domain")
    }
}

impl Basic for VirtIoNetDomain {}

impl DeviceBase for VirtIoNetDomain {
    fn handle_irq(&self) -> AlienResult<()> {
        unimplemented!()
    }
}

static NET: Once<Arc<Mutex<VirtIoNetWrapper>>> = Once::new();

impl NetDomain for VirtIoNetDomain {
    fn init(&self, device_info: &DeviceInfo) -> AlienResult<()> {
        let net = VirtIoNetWrapper::new(device_info);
        let net = Arc::new(Mutex::new(net));
        NET.call_once(|| net);
        Ok(())
    }

    fn mac_address(&self) -> AlienResult<[u8; 6]> {
        Ok(NET.get().unwrap().lock().mac_address())
    }

    fn can_transmit(&self) -> AlienResult<bool> {
        Ok(NET.get().unwrap().lock().can_send())
    }

    fn can_receive(&self) -> AlienResult<bool> {
        Ok(NET.get().unwrap().lock().can_recv())
    }

    fn rx_queue_size(&self) -> AlienResult<usize> {
        Ok(NET_QUEUE_SIZE)
    }

    fn tx_queue_size(&self) -> AlienResult<usize> {
        Ok(NET_QUEUE_SIZE)
    }

    fn recycle_rx_buffer(&self, rx_buf: RxBufferWrapper) -> AlienResult<()> {
        let real_rx_buf = self
            .rx_buf_map
            .lock()
            .remove(&(rx_buf.packet().as_ptr() as usize))
            .unwrap();
        NET.get()
            .unwrap()
            .lock()
            .recycle_rx_buffer(real_rx_buf)
            .unwrap();
        Ok(())
    }

    fn recycle_tx_buffers(&self) -> AlienResult<()> {
        Ok(())
    }

    fn transmit(&self, tx_buf: TxBufferWrapper) -> AlienResult<()> {
        let mut real_tx_buf = self
            .tx_buf_map
            .lock()
            .remove(&(tx_buf.packet().as_ptr() as usize))
            .unwrap();
        real_tx_buf.packet_mut().copy_from_slice(tx_buf.packet());
        NET.get().unwrap().lock().send(real_tx_buf).unwrap();
        Ok(())
    }

    fn receive(&self) -> AlienResult<RxBufferWrapper> {
        let net_buf = NET.get().unwrap().lock().receive().unwrap();
        let shared_buf = RRefVec::from_slice(net_buf.packet());
        self.rx_buf_map
            .lock()
            .insert(shared_buf.as_slice().as_ptr() as usize, net_buf);
        Ok(RxBufferWrapper::new(shared_buf))
    }

    /// Allocate a new buffer for transmitting.
    fn alloc_tx_buffer(&self, size: usize) -> AlienResult<TxBufferWrapper> {
        let buf = NET.get().unwrap().lock().new_tx_buffer(size);
        let shared_buf = RRefVec::new(0, size);
        self.tx_buf_map
            .lock()
            .insert(shared_buf.as_slice().as_ptr() as usize, buf);
        Ok(TxBufferWrapper::new(shared_buf))
    }
}

pub fn main() -> Arc<dyn NetDomain> {
    Arc::new(VirtIoNetDomain::new())
}
