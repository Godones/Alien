use crate::memory::{addr_to_frame, frame_alloc_contiguous};
use core::ptr::NonNull;
use virtio_drivers::{BufferDirection, Hal, PhysAddr, PAGE_SIZE};

pub struct HalImpl;

unsafe impl Hal for HalImpl {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (PhysAddr, NonNull<u8>) {
        let start = frame_alloc_contiguous(pages);
        (start as usize, NonNull::new(start).unwrap())
    }

    unsafe fn dma_dealloc(paddr: PhysAddr, _vaddr: NonNull<u8>, pages: usize) -> i32 {
        for i in 0..pages {
            let frame_tracker = addr_to_frame(paddr + i * PAGE_SIZE);
            drop(frame_tracker);
        }
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
