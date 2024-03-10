#![no_std]

use core::ptr::NonNull;
use virtio_drivers::device::gpu::VirtIOGpu;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
use virtio_drivers::{BufferDirection, Hal, PhysAddr};

pub struct VirtIoGpu {
    gpu: VirtIOGpu<HalImpl, MmioTransport>,
}
unsafe impl Send for VirtIoGpu {}
unsafe impl Sync for VirtIoGpu {}

impl VirtIoGpu {
    pub fn new(base: usize) -> Self {
        let header = NonNull::new(base as *mut VirtIOHeader).unwrap();
        let transport = unsafe { MmioTransport::new(header) }.unwrap();
        let gpu = VirtIOGpu::<HalImpl, MmioTransport>::new(transport)
            .expect("failed to create gpu driver");
        Self { gpu }
    }
    
    pub fn flush(&mut self) -> Result<(), virtio_drivers::Error> {
        self.gpu.flush()
    }
    
}

// Same code to other crates. Will be deleted so simply pasted here.
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
