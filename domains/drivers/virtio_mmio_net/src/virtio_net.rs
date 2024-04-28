use core::{
    ops::{Deref, DerefMut, Range},
    ptr::NonNull,
};

use virtio_drivers::{
    device::net::VirtIONet,
    transport::mmio::{MmioTransport, VirtIOHeader},
};
use virtio_mmio_common::{to_alien_err, HalImpl, SafeIORW};

pub struct VirtIoNetWrapper {
    net: VirtIONet<HalImpl, MmioTransport, NET_QUEUE_SIZE>,
}
unsafe impl Send for VirtIoNetWrapper {}
unsafe impl Sync for VirtIoNetWrapper {}

impl VirtIoNetWrapper {
    pub fn new(address_range: Range<usize>) -> Self {
        let virtio_net_addr = address_range.start;
        basic::println!("virtio_net_addr: {:#x?}", virtio_net_addr);

        let header = NonNull::new(virtio_net_addr as *mut VirtIOHeader).unwrap();
        let transport = unsafe { MmioTransport::new(header) }.unwrap();

        let net = VirtIONet::<HalImpl, MmioTransport, NET_QUEUE_SIZE>::new(transport, NET_BUF_LEN)
            .expect("failed to create gpu driver");
        Self { net }
    }
}

impl Deref for VirtIoNetWrapper {
    type Target = VirtIONet<HalImpl, MmioTransport, NET_QUEUE_SIZE>;

    fn deref(&self) -> &Self::Target {
        &self.net
    }
}

impl DerefMut for VirtIoNetWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.net
    }
}
