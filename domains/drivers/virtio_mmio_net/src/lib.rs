#![no_std]

// mod virtio_net;

extern crate alloc;

use alloc::{boxed::Box, collections::BTreeMap};
use core::{
    fmt::{Debug, Formatter, Result},
    ops::Range,
};

use basic::io::SafeIORegion;
use constants::AlienResult;
use interface::{Basic, DeviceBase, NetDeviceDomain, RxBufferWrapper, TxBufferWrapper};
use ksync::Mutex;
use spin::Once;
use virtio_drivers::{device::net::VirtIONetRaw, transport::mmio::MmioTransport};
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
        unimplemented!()
    }
}

static NET: Once<Mutex<VirtIONetRaw<HalImpl, MmioTransport, NET_QUEUE_SIZE>>> = Once::new();
static RX_MAP: Once<Mutex<BTreeMap<u16, RxBufferWrapper>>> = Once::new();

impl NetDeviceDomain for VirtIoNetDomain {
    fn init(&self, address_range: Range<usize>) -> AlienResult<()> {
        let io_region = SafeIORW(SafeIORegion::from(address_range));
        let transport = MmioTransport::new(Box::new(io_region)).unwrap();
        let mut net = VirtIONetRaw::new(transport).expect("failed to create input driver");

        let mut rx_map = BTreeMap::new();

        for i in 0..NET_QUEUE_SIZE {
            let mut rx = RxBufferWrapper::new(0, NET_BUF_LEN);
            let token = net.receive_begin(rx.packet_mut()).map_err(to_alien_err)?;
            assert_eq!(i, token as _);
            rx_map.insert(token, rx);
        }

        NET.call_once(|| Mutex::new(net));
        RX_MAP.call_once(|| Mutex::new(rx_map));
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

    // fn recycle_rx_buffer(&self, rx_buf: RxBufferWrapper) -> AlienResult<()> {
    //     let real_rx_buf = self
    //         .rx_buf_map
    //         .lock()
    //         .remove(&(rx_buf.packet().as_ptr() as usize))
    //         .unwrap();
    //     NET.get()
    //         .unwrap()
    //         .lock()
    //         .recycle_rx_buffer(real_rx_buf)
    //         .unwrap();
    //     Ok(())
    // }

    // fn recycle_tx_buffers(&self) -> AlienResult<()> {
    //     Ok(())
    // }

    fn transmit(&self, mut tx_buf: TxBufferWrapper) -> AlienResult<()> {
        NET.get()
            .unwrap()
            .lock()
            .send(tx_buf.packet_mut())
            .map_err(to_alien_err)
    }

    // TODO
    fn receive(&self) -> AlienResult<RxBufferWrapper> {
        let mut net = NET.get().unwrap().lock();
        while !net.can_recv().map_err(to_alien_err)?.is_none() {
            core::hint::spin_loop();
        }
        let mut rx_map = RX_MAP.get().unwrap().lock();
        let (token, length) = net.can_recv().map_err(to_alien_err)?.unwrap();
        let res = rx_map.remove(&token).unwrap();
        let (hdr_len, pkt_len) = net.receive_complete(token).map_err(to_alien_err)?;
        assert_eq!(hdr_len + pkt_len, length);

        let mut new_rx = RxBufferWrapper::new(0, NET_BUF_LEN);
        let new_token = net
            .receive_begin(new_rx.packet_mut())
            .map_err(to_alien_err)?;
        assert_eq!(new_token, token);
        assert!(rx_map.insert(new_token, new_rx).is_none());
        Ok(res)
    }

    // Allocate a new buffer for transmitting.
    // fn alloc_tx_buffer(&self, size: usize) -> AlienResult<TxBufferWrapper> {
    //     let buf = NET.get().unwrap().lock().new_tx_buffer(size);
    //     let shared_buf = RRefVec::new(0, size);
    //     self.tx_buf_map
    //         .lock()
    //         .insert(shared_buf.as_slice().as_ptr() as usize, buf);
    //     Ok(TxBufferWrapper::new(shared_buf))
    // }
}

pub fn main() -> Box<dyn NetDeviceDomain> {
    Box::new(VirtIoNetDomain)
}
