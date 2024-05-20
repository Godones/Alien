#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::boxed::Box;

use basic::{io::SafeIORegion, vm::frame::FrameTracker, AlienError};
use virtio_drivers::{
    error::{VirtIoError, VirtIoResult},
    hal::{DevicePage, Hal, QueuePage, VirtIoDeviceIo},
    queue::{QueueLayout, QueueMutRef},
    PhysAddr, VirtAddr,
};

#[derive(Debug)]
pub struct SafeIORW(pub SafeIORegion);

impl VirtIoDeviceIo for SafeIORW {
    fn read_volatile_u32_at(&self, off: usize) -> VirtIoResult<u32> {
        self.0.read_at(off).map_err(|_| VirtIoError::IoError)
    }

    fn read_volatile_u8_at(&self, off: usize) -> VirtIoResult<u8> {
        self.0.read_at(off).map_err(|_| VirtIoError::IoError)
    }

    fn write_volatile_u32_at(&self, off: usize, data: u32) -> VirtIoResult<()> {
        self.0.write_at(off, data).map_err(|_| VirtIoError::IoError)
    }

    fn write_volatile_u8_at(&self, off: usize, data: u8) -> VirtIoResult<()> {
        self.0.write_at(off, data).map_err(|_| VirtIoError::IoError)
    }

    fn paddr(&self) -> PhysAddr {
        self.0.phys_addr().as_usize()
    }

    fn vaddr(&self) -> VirtAddr {
        self.0.virt_addr().as_usize()
    }
}

pub struct Page(FrameTracker);

impl DevicePage for Page {
    fn as_mut_slice(&mut self) -> &mut [u8] {
        self.0.as_mut_slice_with(0)
    }

    fn as_slice(&self) -> &[u8] {
        self.0.as_slice_with(0)
    }

    fn paddr(&self) -> VirtAddr {
        self.0.start_virt_addr().as_usize()
    }

    fn vaddr(&self) -> PhysAddr {
        self.0.start_phy_addr().as_usize()
    }
}

impl<const SIZE: usize> QueuePage<SIZE> for Page {
    fn queue_ref_mut(&mut self, layout: &QueueLayout) -> QueueMutRef<SIZE> {
        let desc_table_offset = layout.descriptor_table_offset;
        let table = self.0.as_mut_slice_with(desc_table_offset);
        let avail_ring_offset = layout.avail_ring_offset;
        let avail_ring = self.0.as_mut_with(avail_ring_offset);

        let used_ring_offset = layout.used_ring_offset;
        let used_ring = self.0.as_mut_with(used_ring_offset);
        QueueMutRef {
            descriptor_table: table,
            avail_ring,
            used_ring,
        }
    }
}

pub struct HalImpl;
impl<const SIZE: usize> Hal<SIZE> for HalImpl {
    fn dma_alloc(pages: usize) -> Box<dyn QueuePage<SIZE>> {
        let frame = FrameTracker::new(pages);
        Box::new(Page(frame))
    }

    fn dma_alloc_buf(pages: usize) -> Box<dyn DevicePage> {
        let frame = FrameTracker::new(pages);
        Box::new(Page(frame))
    }

    fn to_paddr(va: usize) -> usize {
        // println!("<virtio hal> to_paddr: {:#x}", va);
        let va = basic::vaddr_to_paddr_in_kernel(va).expect("vaddr_to_paddr_in_kernel failed");
        va
    }
}

pub fn to_alien_err(e: VirtIoError) -> AlienError {
    log::error!("{:?}", e);
    AlienError::DOMAINCRASH
}
