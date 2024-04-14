#![no_std]

mod gpu;

extern crate alloc;

use alloc::{boxed::Box, collections::BTreeMap, sync::Arc};
use core::{fmt::Debug, ops::Range, ptr::NonNull};

use basic::vm::frame::FrameTracker;
use constants::AlienResult;
use interface::{Basic, DeviceBase, GpuDomain};
use ksync::Mutex;
use rref::RRefVec;
use spin::{Lazy, Once};
use virtio_drivers::{
    device::gpu::VirtIOGpu,
    transport::mmio::{MmioTransport, VirtIOHeader},
    BufferDirection, Hal, PhysAddr,
};

use crate::gpu::VirtIOGpuWrapper;

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
    fn init(&self, address_range: Range<usize>) -> AlienResult<()> {
        let virtio_gpu_addr = address_range.start;
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

pub fn main() -> Box<dyn GpuDomain> {
    Box::new(GPUDomain)
}

pub struct HalImpl;

static PAGE_RECORD: Lazy<Mutex<BTreeMap<usize, FrameTracker>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));

unsafe impl Hal for HalImpl {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (PhysAddr, NonNull<u8>) {
        let frame = FrameTracker::new(pages);
        let phys_addr = frame.start_phy_addr();
        PAGE_RECORD.lock().insert(phys_addr.as_usize(), frame);
        (
            phys_addr.as_usize(),
            NonNull::new(phys_addr.as_usize() as _).unwrap(),
        )
    }

    unsafe fn dma_dealloc(paddr: PhysAddr, _vaddr: NonNull<u8>, _pages: usize) -> i32 {
        let mut page_record = PAGE_RECORD.lock();
        let _frame = page_record.remove(&(paddr)).unwrap();
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: PhysAddr, _size: usize) -> NonNull<u8> {
        let vaddr = PAGE_RECORD.lock().get(&(paddr)).unwrap().start_virt_addr();
        NonNull::new(vaddr.as_usize() as *mut u8).unwrap()
    }

    unsafe fn share(buffer: NonNull<[u8]>, _direction: BufferDirection) -> PhysAddr {
        let vaddr = buffer.as_ptr() as *mut u8 as usize;
        vaddr
    }

    unsafe fn unshare(_paddr: PhysAddr, _buffer: NonNull<[u8]>, _direction: BufferDirection) {}
}
