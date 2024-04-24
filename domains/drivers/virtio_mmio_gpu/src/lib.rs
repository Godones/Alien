#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::boxed::Box;
use core::{fmt::Debug, ops::Range};

use basic::io::SafeIORegion;
use constants::AlienResult;
use interface::{Basic, DeviceBase, GpuDomain};
use ksync::Mutex;
use rref::RRefVec;
use spin::Once;
use virtio_drivers::{device::gpu::VirtIOGpu, transport::mmio::MmioTransport};
use virtio_mmio_common::{HalImpl, SafeIORW};

static GPU: Once<Mutex<VirtIOGpu<HalImpl, MmioTransport>>> = Once::new();

#[derive(Debug)]
pub struct GPUDomain;

impl Basic for GPUDomain {}

impl DeviceBase for GPUDomain {
    fn handle_irq(&self) -> AlienResult<()> {
        unimplemented!()
    }
}

impl GpuDomain for GPUDomain {
    fn init(&self, address_range: Range<usize>) -> AlienResult<()> {
        let virtio_gpu_addr = address_range.start;
        basic::println!("virtio_gpu_addr: {:#x?}", virtio_gpu_addr);
        let io_region = SafeIORW(SafeIORegion::from(address_range));
        let transport = MmioTransport::new(Box::new(io_region)).unwrap();
        let gpu = VirtIOGpu::<HalImpl, MmioTransport>::new(transport)
            .expect("failed to create gpu driver");
        GPU.call_once(|| Mutex::new(gpu));
        Ok(())
    }

    fn flush(&self) -> AlienResult<()> {
        let gpu = GPU.get().unwrap();
        gpu.lock().flush().unwrap();
        Ok(())
    }

    fn fill(&self, _offset: u32, _buf: &RRefVec<u8>) -> AlienResult<usize> {
        todo!()
    }
}

pub fn main() -> Box<dyn GpuDomain> {
    Box::new(GPUDomain)
}
