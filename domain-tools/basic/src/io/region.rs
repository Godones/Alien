use constants::{AlienError, AlienResult};
use core::ops::Range;
use plic::PlicIO;

#[derive(Debug, Copy, Clone)]
pub struct SafeIORegion {
    start: usize,
    size: usize,
}

impl SafeIORegion {
    pub fn new(start: usize, size: usize) -> AlienResult<Self> {
        // check whether the start address is in the kernel space
        // todo!(remove it)
        crate::check_kernel_space(start, size)
            .then(|| ())
            .ok_or(AlienError::EINVAL)?;
        Ok(Self { start, size })
    }

    pub fn from(range: Range<usize>) -> AlienResult<Self> {
        Self::new(range.start, range.end - range.start)
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.start as *const u8, self.size) }
    }

    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.start as *mut u8, self.size) }
    }

    pub fn read_at<T: Copy>(&self, offset: usize) -> AlienResult<T> {
        if offset + core::mem::size_of::<T>() > self.size {
            return Err(AlienError::EINVAL);
        }
        let ptr = (self.start + offset) as *const T;
        unsafe { Ok(ptr.read_volatile()) }
    }

    pub fn write_at<T: Copy>(&self, offset: usize, value: T) -> AlienResult<()> {
        if offset + core::mem::size_of::<T>() > self.size {
            return Err(AlienError::EINVAL);
        }
        let ptr = (self.start + offset) as *mut T;
        unsafe { ptr.write_volatile(value) }
        Ok(())
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
