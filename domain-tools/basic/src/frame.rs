use core::ops::{Deref, DerefMut};
use rref::domain_id;

pub const FRAME_SIZE: usize = 4096;
#[derive(Debug)]
pub struct FrameTracker {
    ptr: usize,
    pages: usize,
}

impl FrameTracker {
    pub fn new(pages: usize) -> Self {
        let ptr = corelib::alloc_raw_pages(pages, domain_id()) as usize;
        Self { ptr, pages }
    }

    /// should be deleted
    pub fn from_raw(ptr: usize, pages: usize) -> Self {
        Self { ptr, pages }
    }

    pub fn end(&self) -> usize {
        self.ptr + self.pages * FRAME_SIZE
    }

    pub fn start(&self) -> usize {
        self.ptr
    }
}

impl Deref for FrameTracker {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { core::slice::from_raw_parts(self.ptr as *const u8, FRAME_SIZE * self.pages) }
    }
}

impl DerefMut for FrameTracker {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::slice::from_raw_parts_mut(self.ptr as *mut u8, FRAME_SIZE * self.pages) }
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        corelib::free_raw_pages(self.ptr as *mut u8, self.pages, domain_id());
    }
}
