use interface::{Basic, DeviceBase, GpuDomain};
use rref::{RRef, RRefVec, RpcResult};
use std::sync::Arc;
#[derive(Debug)]
pub struct GPUDomain {}

impl GPUDomain {
    fn new(virtio_gpu_addr: usize) -> Self {
        Self {}
    }
}

impl Basic for GPUDomain {}

impl DeviceBase for GPUDomain {
    fn handle_irq(&self) -> RpcResult<()> {
        unimplemented!()
    }
}

impl GpuDomain for GPUDomain {
    fn flush(&self) -> RpcResult<()> {
        todo!()
    }

    fn fill(&self, offset: u32, buf: &RRefVec<u8>) -> RpcResult<()> {
        todo!()
    }
}

pub fn main(virtio_gpu_addr: usize) -> Arc<dyn GpuDomain> {
    libsyscall::println!("virtio_gpu_addr: {:#x}", virtio_gpu_addr);
    Arc::new(GPUDomain::new(virtio_gpu_addr))
}
