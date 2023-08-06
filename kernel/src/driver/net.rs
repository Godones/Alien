use core::ptr::NonNull;

use virtio_drivers::device::net::VirtIONet;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};

use crate::driver::hal::HalImpl;

pub const NET_BUFFER_LEN: usize = 65536 / 2;
pub const NET_QUEUE_SIZE: usize = 64;

type VirtIONetDevice = VirtIONet<HalImpl, MmioTransport, NET_QUEUE_SIZE>;

pub struct VirtIONetDeviceWrapper {
    net: Option<VirtIONetDevice>,
}

impl VirtIONetDeviceWrapper {
    pub fn from_addr(addr: usize) -> Self {
        let header = NonNull::new(addr as *mut VirtIOHeader).unwrap();
        let transport = unsafe { MmioTransport::new(header) }.unwrap();
        let net =
            VirtIONet::<HalImpl, MmioTransport, NET_QUEUE_SIZE>::new(transport, NET_BUFFER_LEN)
                .expect("failed to create net driver");
        Self { net: Some(net) }
    }
    pub fn take(&mut self) -> Option<VirtIONetDevice> {
        self.net.take()
    }
}
