use core::ops::Range;

use downcast_rs::{impl_downcast, DowncastSync};
use gproxy::proxy;
use rref::RRefVec;

use super::AlienResult;
use crate::{Basic, DeviceBase};

#[proxy(NetDeviceDomainProxy, Range<usize>)]
pub trait NetDeviceDomain: DeviceBase + Basic + DowncastSync {
    fn init(&self, device_info: Range<usize>) -> AlienResult<()>;
    /// The ethernet address of the NIC.
    fn mac_address(&self) -> AlienResult<[u8; 6]>;

    /// Whether can transmit packets.
    fn can_transmit(&self) -> AlienResult<bool>;

    /// Whether can receive packets.
    fn can_receive(&self) -> AlienResult<bool>;

    /// Size of the receive queue.
    fn rx_queue_size(&self) -> AlienResult<usize>;

    /// Size of the transmit queue.
    fn tx_queue_size(&self) -> AlienResult<usize>;

    /// Transmits a packet in the buffer to the network, without blocking,
    /// returns [`DevResult`].
    fn transmit(&self, tx_buf: &RRefVec<u8>) -> AlienResult<()>;

    /// Receives a packet from the network and store it in the [`NetBuf`],
    /// returns the buffer.
    ///
    /// Before receiving, the driver should have already populated some buffers
    /// in the receive queue by [`NetDriverOps::recycle_rx_buffer`].
    ///
    /// If currently no incomming packets, returns an error with type
    /// [`DevError::Again`].
    fn receive(&self, rx_buf: RRefVec<u8>) -> AlienResult<(RRefVec<u8>, usize)>;
}

impl_downcast!(sync NetDeviceDomain);
