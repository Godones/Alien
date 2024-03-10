#![no_std]
extern crate alloc;
extern crate malloc;

use interface::{Basic, DeviceBase, GpuDomain, DeviceInfo};
use rref::{RRef, RRefVec, RpcError, RpcResult};
use virtio_gpu::VirtIoGpu;
use ksync::Mutex;
use alloc::sync::Arc;
use core::{fmt::Debug, 
    result::Result::{Ok, Err}, 
    concat, format_args, todo, unimplemented, write};


pub struct GPUDomain {
    driver: Arc<Mutex<VirtIoGpu>>,
}

impl GPUDomain {
    fn new(virtio_gpu_addr: usize) -> Self {
        Self {
            driver: Arc::new(Mutex::new(VirtIoGpu::new(virtio_gpu_addr)))
        }
    }
}

impl Basic for GPUDomain {}

impl Debug for GPUDomain {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Gpu Domain (virtio)")
    } 
}

impl DeviceBase for GPUDomain {
    fn handle_irq(&self) -> RpcResult<()> {
        unimplemented!()
    }
}

impl GpuDomain for GPUDomain {
    fn flush(&self) -> RpcResult<()> {
        match self.driver.lock().flush() {
            Ok(_) => Ok(()),
            Err(e) => todo!(),
        }
    }

    fn fill(&self, offset: u32, buf: &RRefVec<u8>) -> RpcResult<usize> {
        todo!()
    }
}

pub fn main() -> Arc<dyn GpuDomain> {
    let devices_domain = libsyscall::get_devices_domain().unwrap();
    let name = RRefVec::from_slice("virtio-mmio-gpu".as_bytes());

    let info = RRef::new(DeviceInfo {
        address_range: Default::default(),
        irq: RRef::new(0),
        compatible: RRefVec::new(0, 64),
    });

    let info = devices_domain.get_device(name, info).unwrap();

    let virtio_gpu_addr = &info.address_range;

    libsyscall::println!("virtio_gpu_addr: {:#x?}", virtio_gpu_addr);
    Arc::new(GPUDomain::new(virtio_gpu_addr.start))
}
