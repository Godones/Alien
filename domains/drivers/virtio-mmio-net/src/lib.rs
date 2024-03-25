#![no_std]

mod virtio_net;

extern crate alloc;

use core::fmt::{Debug, Formatter, Result};
use alloc::sync::Arc;
use constants::AlienResult;
use interface::{Basic, DeviceBase, DeviceInfo, NetDomain};
use rref::RRefVec;
use spin::Once;
use ksync::Mutex;

use virtio_drivers::device::net::RxBuffer;
use virtio_net::{VirtIoNetWrapper, NET_QUEUE_SIZE};

pub struct VirtIoNetDomain;

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

    fn mac_address(&self) -> AlienResult<[u8;6]> {
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

    fn recycle_rx_buffer(&self, rx_buf: RRefVec<u8>) -> AlienResult<()> {
        NET.get().unwrap().lock().recycle_rx_buffer(RxBuffer)
    }

    fn recycle_tx_buffers(&self) -> AlienResult<()> {
        Ok(())
    }

    fn transmit(&self, data: RRefVec<u8>) -> AlienResult<()> {
        todo!()
    }

    fn receive(&self) -> AlienResult<RRefVec<u8>> {
        todo!()
    }

    fn alloc_tx_buffer(&self, size: usize) -> AlienResult<RRefVec<u8>> {
        todo!()
    }
}

pub fn main() -> Arc<dyn NetDomain> {
    Arc::new(VirtIoNetDomain)
}
