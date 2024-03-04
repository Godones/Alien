//! This crate should provide abstractions related to virtio registers and buffers,
//! but should not provide more functionality. It acts as part of the kernel's trusted code,
//! so unsafe code can be used.
#![no_std]

extern crate alloc;

use core::ptr::NonNull;
use virtio_drivers::device::blk::VirtIOBlk;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
use virtio_drivers::{BufferDirection, Hal, PhysAddr};

/// For simplicity, now we directly use the `virtio-drivers` crate.
///
/// In the future, we should implement our own virtio driver.
pub struct VirtIoBlk {
    device: VirtIOBlk<HalImpl, MmioTransport>,
}
unsafe impl Send for VirtIoBlk {}
unsafe impl Sync for VirtIoBlk {}

impl VirtIoBlk {
    pub fn new(base: usize) -> Self {
        let header = NonNull::new(base as *mut VirtIOHeader).unwrap();
        let transport = unsafe { MmioTransport::new(header) }.unwrap();
        let blk = VirtIOBlk::<HalImpl, MmioTransport>::new(transport)
            .expect("failed to create blk driver");
        Self { device: blk }
    }
}

impl VirtIoBlk {
    pub fn read_block(&mut self, block_id: usize, buf: &mut [u8]) -> Result<usize, &'static str> {
        self.device
            .read_block(block_id, buf)
            .map_err(|_| "failed to read block")?;
        Ok(buf.len())
    }
    pub fn write_block(&mut self, block_id: usize, buf: &[u8]) -> Result<usize, &'static str> {
        self.device
            .write_block(block_id, buf)
            .map_err(|_| "failed to write block")?;
        Ok(buf.len())
    }

    pub fn capacity(&self) -> usize {
        self.device.capacity() as usize * 512
    }
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
