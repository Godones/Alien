#![no_std]

mod gpu;

extern crate alloc;

use crate::gpu::VirtIOGpuWrapper;
use alloc::sync::Arc;
use basic::frame::FrameTracker;
use constants::AlienResult;
use core::fmt::Debug;
use core::mem::forget;
use core::ptr::NonNull;
use interface::{Basic, DeviceBase, DeviceInfo, GpuDomain};
use ksync::Mutex;
use rref::RRefVec;
use spin::Once;
use virtio_drivers::device::gpu::VirtIOGpu;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
use virtio_drivers::{BufferDirection, Hal, PhysAddr};

static GPU: Once<Arc<Mutex<VirtIOGpuWrapper>>> = Once::new();

#[derive(Debug)]
pub struct GPUDomain;

impl Basic for GPUDomain {}

impl DeviceBase for GPUDomain {
    fn handle_irq(&self) -> AlienResult<()> {
        unimplemented!()
    }
}

impl GpuDomain for GPUDomain {
    fn init(&self, device_info: &DeviceInfo) -> AlienResult<()> {
        let virtio_gpu_addr = device_info.address_range.start;
        basic::println!("virtio_gpu_addr: {:#x?}", virtio_gpu_addr);

        let header = NonNull::new(virtio_gpu_addr as *mut VirtIOHeader).unwrap();
        let transport = unsafe { MmioTransport::new(header) }.unwrap();

        let gpu = VirtIOGpu::<HalImpl, MmioTransport>::new(transport)
            .expect("failed to create gpu driver");

        let gpu = Arc::new(Mutex::new(VirtIOGpuWrapper::new(gpu)));
        GPU.call_once(|| gpu);
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

pub fn main() -> Arc<dyn GpuDomain> {
    Arc::new(GPUDomain)
}

pub struct HalImpl;

unsafe impl Hal for HalImpl {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (PhysAddr, NonNull<u8>) {
        let frame = FrameTracker::new(pages);
        let ptr = frame.start();
        forget(frame);
        (ptr, NonNull::new(ptr as _).unwrap())
    }

    unsafe fn dma_dealloc(paddr: PhysAddr, _vaddr: NonNull<u8>, pages: usize) -> i32 {
        let _frame = FrameTracker::from_raw(paddr, pages);
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: PhysAddr, _size: usize) -> NonNull<u8> {
        NonNull::new(paddr as *mut u8).unwrap()
    }

    unsafe fn share(buffer: NonNull<[u8]>, _direction: BufferDirection) -> PhysAddr {
        let vaddr = buffer.as_ptr() as *mut u8 as usize;
        vaddr
    }

    unsafe fn unshare(_paddr: PhysAddr, _buffer: NonNull<[u8]>, _direction: BufferDirection) {}
}
