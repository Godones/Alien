use core::ops::{Deref, DerefMut};

use config::FRAME_SIZE;
use memory_addr::{PhysAddr, VirtAddr};
use rref::domain_id;

#[derive(Debug)]
pub struct FrameTracker {
    ptr: usize,
    page_count: usize,
    // should be deallocated
    dealloc: bool,
}

impl FrameTracker {
    /// Allocate `page_count` pages and return a `FrameTracker` pointing to the start of the allocated memory.
    pub fn new(page_count: usize) -> Self {
        let ptr = corelib::alloc_raw_pages(page_count, domain_id()) as usize;
        Self {
            ptr,
            page_count,
            dealloc: true,
        }
    }

    pub fn create_trampoline() -> Self {
        let trampoline_phy_addr = corelib::trampoline_addr();
        Self {
            ptr: trampoline_phy_addr,
            page_count: 1,
            dealloc: false,
        }
    }

    /// Return the physical address of the start of the frame.
    pub fn start_phy_addr(&self) -> PhysAddr {
        PhysAddr::from(self.ptr)
    }

    /// Return the virtual address of the start of the frame.
    pub fn start_virt_addr(&self) -> VirtAddr {
        VirtAddr::from(self.ptr)
    }

    /// Return the physical address of the end of the frame.
    pub fn end_phy_addr(&self) -> PhysAddr {
        PhysAddr::from(self.end())
    }

    /// Return the virtual address of the end of the frame.
    pub fn end_virt_addr(&self) -> VirtAddr {
        VirtAddr::from(self.end())
    }

    fn end(&self) -> usize {
        self.ptr + self.size()
    }

    pub fn size(&self) -> usize {
        self.page_count * FRAME_SIZE
    }

    pub fn clear(&self) {
        unsafe {
            core::ptr::write_bytes(self.ptr as *mut u8, 0, self.size());
        }
    }
    pub fn as_mut_slice_with<'a, T>(&self) -> &'a mut [T] {
        assert_eq!(FRAME_SIZE % core::mem::size_of::<T>(), 0);
        unsafe {
            core::slice::from_raw_parts_mut(
                self.ptr as *mut T,
                self.size() / core::mem::size_of::<T>(),
            )
        }
    }
    pub fn as_slice_with<'a, T>(&self) -> &'a [T] {
        let size = core::mem::size_of::<T>();
        assert_eq!(FRAME_SIZE % size, 0);
        unsafe { core::slice::from_raw_parts(self.ptr as *const T, self.size() / size) }
    }
}

impl Deref for FrameTracker {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { core::slice::from_raw_parts(self.ptr as *const u8, self.size()) }
    }
}

impl DerefMut for FrameTracker {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::slice::from_raw_parts_mut(self.ptr as *mut u8, self.size()) }
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        if self.dealloc {
            corelib::free_raw_pages(self.ptr as *mut u8, self.page_count, domain_id());
        }
    }
}
