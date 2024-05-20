#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::boxed::Box;
use core::{
    fmt::{Debug, Formatter, Result},
    ops::Range,
};

use basic::{io::SafeIORegion, sync::Mutex, AlienResult};
use interface::{Basic, DeviceBase, NetDeviceDomain};
use rref::RRefVec;
use spin::Once;
use virtio_drivers::{device::net::VirtIONet, transport::mmio::MmioTransport};
use virtio_mmio_common::{to_alien_err, HalImpl, SafeIORW};

pub const NET_QUEUE_SIZE: usize = 128;
pub const NET_BUF_LEN: usize = 4096;

pub struct VirtIoNetDomain;

impl Debug for VirtIoNetDomain {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        writeln!(f, "Net Domain")
    }
}

impl Basic for VirtIoNetDomain {}

impl DeviceBase for VirtIoNetDomain {
    fn handle_irq(&self) -> AlienResult<()> {
        log::info!("<VirtIoNetDomain as DeviceBase>::handle_irq() called");
        NET.get()
            .unwrap()
            .lock()
            .ack_interrupt()
            .map_err(to_alien_err)?;
        Ok(())
    }
}

static NET: Once<Mutex<VirtIONet<HalImpl, MmioTransport, NET_QUEUE_SIZE>>> = Once::new();
pub const NET_BUFFER_LEN: usize = 1600;

impl NetDeviceDomain for VirtIoNetDomain {
    fn init(&self, address_range: Range<usize>) -> AlienResult<()> {
        let io_region = SafeIORW(SafeIORegion::from(address_range));
        let transport = MmioTransport::new(Box::new(io_region)).unwrap();
        let net = VirtIONet::new(transport, NET_BUFFER_LEN).expect("failed to create input driver");
        NET.call_once(|| Mutex::new(net));
        Ok(())
    }

    fn mac_address(&self) -> AlienResult<[u8; 6]> {
        NET.get()
            .unwrap()
            .lock()
            .mac_address()
            .map_err(to_alien_err)
    }

    fn can_transmit(&self) -> AlienResult<bool> {
        NET.get().unwrap().lock().can_send().map_err(to_alien_err)
    }

    fn can_receive(&self) -> AlienResult<bool> {
        Ok(NET
            .get()
            .unwrap()
            .lock()
            .can_recv()
            .map_err(to_alien_err)?
            .is_some())
    }

    fn rx_queue_size(&self) -> AlienResult<usize> {
        Ok(NET_QUEUE_SIZE)
    }

    fn tx_queue_size(&self) -> AlienResult<usize> {
        Ok(NET_QUEUE_SIZE)
    }

    fn transmit(&self, tx_buf: &RRefVec<u8>) -> AlienResult<()> {
        NET.get()
            .unwrap()
            .lock()
            .send(tx_buf.as_slice())
            .map_err(to_alien_err)
    }

    fn receive(&self, mut rx_buf: RRefVec<u8>) -> AlienResult<(RRefVec<u8>, usize)> {
        let len = NET
            .get()
            .unwrap()
            .lock()
            .receive(rx_buf.as_mut_slice())
            .map_err(to_alien_err)?;
        Ok((rx_buf, len))
    }
}

pub fn main() -> Box<dyn NetDeviceDomain> {
    Box::new(VirtIoNetDomain)
}
