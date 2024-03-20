use crate::HalImpl;
use core::fmt::Debug;
use core::ops::{Deref, DerefMut};
use virtio_drivers::device::gpu::VirtIOGpu;
use virtio_drivers::transport::mmio::MmioTransport;

pub struct VirtIOGpuWrapper {
    gpu: VirtIOGpu<HalImpl, MmioTransport>,
}

unsafe impl Send for VirtIOGpuWrapper {}

unsafe impl Sync for VirtIOGpuWrapper {}

impl Debug for VirtIOGpuWrapper {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "VirtIOGpuWrapper")
    }
}

impl VirtIOGpuWrapper {
    pub fn new(gpu: VirtIOGpu<HalImpl, MmioTransport>) -> Self {
        Self { gpu }
    }
}

impl Deref for VirtIOGpuWrapper {
    type Target = VirtIOGpu<HalImpl, MmioTransport>;

    fn deref(&self) -> &Self::Target {
        &self.gpu
    }
}

impl DerefMut for VirtIOGpuWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.gpu
    }
}
