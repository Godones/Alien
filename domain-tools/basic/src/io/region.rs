use core::ops::Range;

use constants::{AlienError, AlienResult};
use goldfish_rtc::GoldFishRtcIo;
use memory_addr::{PhysAddr, VirtAddr};
use raw_plic::PlicIO;
use raw_uart16550::Uart16550IO;

#[derive(Debug, Clone)]
pub struct SafeIORegion {
    range: Range<PhysAddr>,
}

impl From<Range<usize>> for SafeIORegion {
    fn from(value: Range<usize>) -> Self {
        let start = PhysAddr::from(value.start);
        let end = PhysAddr::from(value.end);
        Self { range: start..end }
    }
}

impl SafeIORegion {
    pub fn new(range: Range<PhysAddr>) -> Self {
        Self { range }
    }

    pub fn as_bytes(&self) -> &[u8] {
        let start = self.range.start.as_usize();
        unsafe { core::slice::from_raw_parts(start as *const u8, self.size()) }
    }

    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        let start = self.range.start.as_usize();
        unsafe { core::slice::from_raw_parts_mut(start as *mut u8, self.size()) }
    }

    pub fn read_at<T: Copy>(&self, offset: usize) -> AlienResult<T> {
        if offset + core::mem::size_of::<T>() > self.size() {
            return Err(AlienError::EINVAL);
        }
        let start = self.range.start.as_usize();
        let ptr = (start + offset) as *const T;
        unsafe { Ok(ptr.read_volatile()) }
    }

    pub fn write_at<T: Copy>(&self, offset: usize, value: T) -> AlienResult<()> {
        if offset + core::mem::size_of::<T>() > self.size() {
            return Err(AlienError::EINVAL);
        }
        let start = self.range.start.as_usize();
        let ptr = (start + offset) as *mut T;
        unsafe { ptr.write_volatile(value) }
        Ok(())
    }

    pub fn phys_addr(&self) -> PhysAddr {
        self.range.start
    }

    pub fn virt_addr(&self) -> VirtAddr {
        VirtAddr::from(self.range.start.as_usize())
    }

    pub fn size(&self) -> usize {
        self.range.end.as_usize() - self.range.start.as_usize()
    }
}

impl PlicIO for SafeIORegion {
    fn read_at(&self, offset: usize) -> AlienResult<u32> {
        SafeIORegion::read_at(self, offset)
    }
    fn write_at(&self, offset: usize, value: u32) -> AlienResult<()> {
        SafeIORegion::write_at(self, offset, value)
    }
}

impl GoldFishRtcIo for SafeIORegion {
    fn read_at(&self, offset: usize) -> AlienResult<u32> {
        SafeIORegion::read_at(self, offset)
    }
    fn write_at(&self, offset: usize, value: u32) -> AlienResult<()> {
        SafeIORegion::write_at(self, offset, value)
    }
}

impl Uart16550IO for SafeIORegion {
    fn read_at(&self, offset: usize) -> AlienResult<u8> {
        SafeIORegion::read_at(self, offset)
    }

    fn write_at(&self, offset: usize, value: u8) -> AlienResult<()> {
        SafeIORegion::write_at(self, offset, value)
    }
}
