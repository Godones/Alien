#![no_std]

mod virtio_net;

extern crate alloc;

use core::fmt::{Debug, Formatter, Result};
use alloc::{boxed::Box, sync::Arc};
use constants::AlienResult;
use interface::{Basic, DeviceBase, DeviceInfo, NetDomain, NetBuf};
use rref::RRefVec;
use spin::Once;
use ksync::Mutex;

pub use virtio_drivers::device::net::{RxBuffer, TxBuffer};
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

    fn recycle_rx_buffer(&self, net_buf: NetBuf) -> AlienResult<()> {
        let rx_buf = net_buf.net_buf;
        let rx_buf = rx_buf.downcast::<RxBuffer>().unwrap();
        NET.get().unwrap().lock().recycle_rx_buffer(*rx_buf).unwrap();
        Ok(())
    }

    fn recycle_tx_buffers(&self) -> AlienResult<()> {
        Ok(())
    }

    fn transmit(&self, net_buf: NetBuf) -> AlienResult<()> {
        let tx_buf = net_buf.net_buf;
        let tx_buf = tx_buf.downcast::<TxBuffer>().unwrap();
        NET.get().unwrap().lock().send(*tx_buf).unwrap();
        Ok(())
    }

    fn receive(&self) -> AlienResult<NetBuf> {
        let net_buf = NET.get().unwrap().lock().receive().unwrap();
        let data = RRefVec::from_slice(net_buf.packet());

        Ok(NetBuf {
            data,
            net_buf: Box::new(net_buf)
        })
    }

    fn alloc_tx_buffer(&self, size: usize) -> AlienResult<NetBuf> {
        let buf = NET.get().unwrap().lock().new_tx_buffer(size);
        Ok(NetBuf {
            data: RRefVec::new(0, size),
            net_buf: Box::new(buf),
        })
    }
}

pub fn main() -> Arc<dyn NetDomain> {
    Arc::new(VirtIoNetDomain)
}