use constants::{AlienError, AlienResult};

#[derive(Debug, Copy)]
pub struct SafeRegion {
    start: usize,
    size: usize,
}

impl SafeRegion {
    pub fn new(start: usize, size: usize) -> AlienResult<Self> {
        // check whether the start address is in the kernel space
        mem::is_in_kernel_space(start, size)
            .then(|| ())
            .ok_or(AlienError::EINVAL)?;
        Ok(Self { start, size })
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
        Ok(unsafe { *ptr }.clone())
    }
    pub fn write_at<T: Copy>(&mut self, offset: usize, value: T) -> AlienResult<()> {
        if offset + core::mem::size_of::<T>() > self.size {
            return Err(AlienError::EINVAL);
        }
        let ptr = (self.start + offset) as *mut T;
        unsafe {
            *ptr = value;
        }
        Ok(())
    }
}
