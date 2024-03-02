use core::ops::{Deref, DerefMut};

pub const FRAME_SIZE: usize = 4196;
#[derive(Debug)]
pub struct FrameTracker {
    ptr: usize,
    count: usize,
}

impl FrameTracker {
    pub fn new(ptr: usize, count: usize) -> Self {
        Self { ptr, count }
    }
}

impl Deref for FrameTracker {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { core::slice::from_raw_parts(self.ptr as *const u8, FRAME_SIZE * self.count) }
    }
}

impl DerefMut for FrameTracker {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::slice::from_raw_parts_mut(self.ptr as *mut u8, FRAME_SIZE * self.count) }
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        // todo!(change interface)
        crate::free_raw_pages(self.ptr as *mut u8, self.count);
    }
}
