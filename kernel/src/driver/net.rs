use core::ptr::NonNull;

use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
use virtio_net::VirtIONetDeviceWrapper;

use crate::driver::hal::HalImpl;

pub const NET_BUFFER_LEN: usize = 4096;
pub const NET_QUEUE_SIZE: usize = 128;

pub fn make_virtio_net_device(
    addr: usize,
) -> VirtIONetDeviceWrapper<HalImpl, MmioTransport, NET_QUEUE_SIZE> {
    let header = NonNull::new(addr as *mut VirtIOHeader).unwrap();
    let transport = unsafe { MmioTransport::new(header) }.unwrap();
    let device = VirtIONetDeviceWrapper::new(transport, NET_BUFFER_LEN);
    device
}
