use crate::memory::{addr_to_frame, frames_alloc};
use core::intrinsics::forget;
use core::ptr::NonNull;
use pci::PortOps;
use virtio_drivers::{BufferDirection, Hal, PhysAddr, PAGE_SIZE};

pub struct HalImpl;

unsafe impl Hal for HalImpl {
    fn dma_alloc(pages: usize, direction: BufferDirection) -> (PhysAddr, NonNull<u8>) {
        let addr = frames_alloc(pages);
        let start = addr.as_ref().unwrap()[0].start();
        forget(addr);
        (start, NonNull::new(start as *mut u8).unwrap())
    }

    unsafe fn dma_dealloc(paddr: PhysAddr, vaddr: NonNull<u8>, pages: usize) -> i32 {
        for i in 0..pages {
            let frame_tracker = addr_to_frame(paddr + i * PAGE_SIZE);
            drop(frame_tracker);
        }
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: PhysAddr, size: usize) -> NonNull<u8> {
        NonNull::new(paddr as *mut u8).unwrap()
    }

    unsafe fn share(buffer: NonNull<[u8]>, direction: BufferDirection) -> PhysAddr {
        let vaddr = buffer.as_ptr() as *mut u8 as usize;
        vaddr
    }

    unsafe fn unshare(paddr: PhysAddr, buffer: NonNull<[u8]>, direction: BufferDirection) {}
}



pub struct PortImpl;
impl PortOps for PortImpl{
    unsafe fn read8(&self, port: u16) -> u8 {
        0
    }

    unsafe fn read16(&self, port: u16) -> u16 {
        0
    }

    unsafe fn read32(&self, port: u32) -> u32 {
        0
    }

    unsafe fn write8(&self, port: u16, val: u8) {}

    unsafe fn write16(&self, port: u16, val: u16) {}

    unsafe fn write32(&self, port: u32, val: u32) {}
}