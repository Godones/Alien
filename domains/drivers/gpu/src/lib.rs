use interface::{Basic, Gpu};
use rref::{RRef, RpcResult};
use std::sync::Arc;
#[derive(Debug)]
pub struct GPUDomain {}

impl GPUDomain {
    fn new(virtio_gpu_addr: usize) -> Self {
        Self {}
    }
}

impl Basic for GPUDomain {}

impl Gpu for GPUDomain {
    fn flush(&self) -> RpcResult<()> {
        todo!()
    }

    fn fill_buf(&self, buf: RRef<[u8; 1280 * 800]>) -> RpcResult<()> {
        todo!()
    }
}

pub fn main(virtio_gpu_addr: usize) -> Arc<dyn Gpu> {
    libsyscall::println!("virtio_gpu_addr: {:#x}", virtio_gpu_addr);
    Arc::new(GPUDomain::new(virtio_gpu_addr))
}
