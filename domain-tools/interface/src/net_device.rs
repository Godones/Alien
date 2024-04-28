use core::ops::Range;

use constants::AlienResult;
use downcast_rs::{impl_downcast, DowncastSync};
use gproxy::proxy;
use rref::RRefVec;

use crate::{Basic, DeviceBase};
#[derive(Debug)]
pub struct PackageBuffer(RRefVec<u8>);

pub type TxBufferWrapper = PackageBuffer;
pub type RxBufferWrapper = PackageBuffer;

impl PackageBuffer {
    /// Construct a new buffer
    pub fn new(initial_value: u8, size: usize) -> Self {
        Self(RRefVec::new(initial_value, size))
    }

    /// Constructs the buffer from the given slice.
    pub fn from(buf: RRefVec<u8>) -> Self {
        Self(buf)
    }

    /// Returns the network packet length.
    pub fn packet_len(&self) -> usize {
        self.0.len()
    }

    /// Returns the network packet as a slice.
    pub fn packet(&self) -> &[u8] {
        self.0.as_slice()
    }

    /// Returns the network packet as a mutable slice.
    pub fn packet_mut(&mut self) -> &mut [u8] {
        self.0.as_mut_slice()
    }
}

#[proxy(NetDeviceDomainProxy, Range<usize>)]
pub trait NetDeviceDomain: DeviceBase + Basic + DowncastSync {
    fn init(&self, device_info: Range<usize>) -> AlienResult<()>;
    // fn medium(&self) -> Medium;

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

    // /// Gives back the `rx_buf` to the receive queue for later receiving.
    // ///
    // /// `rx_buf` should be the same as the one returned by
    // /// [`NetDriverOps::receive`].
    // fn recycle_rx_buffer(&self, rx_buf: RxBufferWrapper) -> AlienResult<()>;

    /// Poll the transmit queue and gives back the buffers for previous transmiting.
    /// returns [`DevResult`].
    // fn recycle_tx_buffers(&self) -> AlienResult<()>;

    /// Transmits a packet in the buffer to the network, without blocking,
    /// returns [`DevResult`].
    fn transmit(&self, tx_buf: TxBufferWrapper) -> AlienResult<()>;

    /// Receives a packet from the network and store it in the [`NetBuf`],
    /// returns the buffer.
    ///
    /// Before receiving, the driver should have already populated some buffers
    /// in the receive queue by [`NetDriverOps::recycle_rx_buffer`].
    ///
    /// If currently no incomming packets, returns an error with type
    /// [`DevError::Again`].
    fn receive(&self) -> AlienResult<RxBufferWrapper>;

    // /// Allocate a memory buffer of a specified size for network transmission,
    // /// returns [`DevResult`]
    // fn alloc_tx_buffer(&self, size: usize) -> AlienResult<TxBufferWrapper>;
}

impl_downcast!(sync NetDeviceDomain);
