use crate::memory::{addr_to_frame, frames_alloc};
use core::intrinsics::forget;
use virtio_drivers::{Hal, PhysAddr, VirtAddr, PAGE_SIZE};

pub struct HalImpl;

impl Hal for HalImpl {
    fn dma_alloc(pages: usize) -> PhysAddr {
        let addr = frames_alloc(pages);
        let start = addr.as_ref().unwrap()[0].start();
        forget(addr);
        start
    }

    fn dma_dealloc(paddr: PhysAddr, pages: usize) -> i32 {
        for i in 0..pages {
            let frame_tracker = addr_to_frame(paddr + i * PAGE_SIZE);
            drop(frame_tracker);
        }
        0
    }

    fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
        paddr
    }

    fn virt_to_phys(vaddr: VirtAddr) -> PhysAddr {
        vaddr
    }
}
