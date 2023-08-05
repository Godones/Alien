use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;

use virtio_drivers::device::net::VirtIONet;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};

use crate::driver::hal::HalImpl;

pub struct VirtIONetDeviceWrapper<const QS: usize, const BL: usize> {
    net: VirtIONet<HalImpl, MmioTransport, QS>,
}

impl<const QS: usize, const BL: usize> VirtIONetDeviceWrapper<QS, BL> {
    pub fn from_addr(addr: usize) -> Self {
        let header = NonNull::new(addr as *mut VirtIOHeader).unwrap();
        let transport = unsafe { MmioTransport::new(header) }.unwrap();
        let net = VirtIONet::<HalImpl, MmioTransport, QS>::new(transport, BL)
            .expect("failed to create net driver");
        Self { net }
    }
}

impl<const QS: usize, const BL: usize> Deref for VirtIONetDeviceWrapper<QS, BL> {
    type Target = VirtIONet<HalImpl, MmioTransport, QS>;

    fn deref(&self) -> &Self::Target {
        &self.net
    }
}

impl<const QS: usize, const BL: usize> DerefMut for VirtIONetDeviceWrapper<QS, BL> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.net
    }
}
