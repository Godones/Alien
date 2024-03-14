#![no_std]
extern crate alloc;
extern crate malloc;

use alloc::sync::Arc;
use core::ptr::NonNull;
use core::{
    fmt::Debug,
    result::Result::{Err, Ok},
    todo, unimplemented, write,
};
use interface::{Basic, DeviceBase, DeviceInfo, GpuDomain};
use ksync::Mutex;
use rref::{RRef, RRefVec, RpcResult};
use virtio_drivers::device::gpu::VirtIOGpu;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
use virtio_drivers::{BufferDirection, Hal, PhysAddr};

pub struct GPUDomain {
    gpu: Arc<Mutex<VirtIOGpu<HalImpl, MmioTransport>>>,
}
unsafe impl Send for GPUDomain {}
unsafe impl Sync for GPUDomain {}

impl GPUDomain {
    fn new(virtio_gpu_addr: usize) -> Self {
        let header = NonNull::new(virtio_gpu_addr as *mut VirtIOHeader).unwrap();
        let transport = unsafe { MmioTransport::new(header) }.unwrap();
        Self {
            gpu: Arc::new(Mutex::new(VirtIOGpu::<HalImpl, MmioTransport>::new(transport).expect("failed to create gpu driver"))),
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
        match self.gpu.lock().flush() {
            Ok(_) => Ok(()),
            Err(_) => todo!(),
        }
    }

    fn fill(&self, _offset: u32, _buf: &RRefVec<u8>) -> RpcResult<usize> {
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

struct HalImpl;

unsafe impl Hal for HalImpl {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (PhysAddr, NonNull<u8>) {
        let ptr = libsyscall::alloc_raw_pages(pages);
        (ptr as usize, NonNull::new(ptr).unwrap())
    }

    unsafe fn dma_dealloc(paddr: PhysAddr, _vaddr: NonNull<u8>, pages: usize) -> i32 {
        libsyscall::free_raw_pages(paddr as *mut u8, pages);
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
