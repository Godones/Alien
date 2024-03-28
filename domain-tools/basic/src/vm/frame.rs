use core::ops::{Deref, DerefMut};

use config::FRAME_SIZE;
use memory_addr::{PhysAddr, VirtAddr};
use page_table::{NotLeafPage, Rv64PTE, ENTRY_COUNT};
use ptable::PhysPage;
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
        self.ptr + self.page_count * FRAME_SIZE
    }

    fn start(&self) -> usize {
        self.ptr
    }
}

impl Deref for FrameTracker {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { core::slice::from_raw_parts(self.ptr as *const u8, FRAME_SIZE * self.page_count) }
    }
}

impl DerefMut for FrameTracker {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            core::slice::from_raw_parts_mut(self.ptr as *mut u8, FRAME_SIZE * self.page_count)
        }
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        if self.dealloc {
            corelib::free_raw_pages(self.ptr as *mut u8, self.page_count, domain_id());
        }
    }
}

impl NotLeafPage<Rv64PTE> for FrameTracker {
    fn phys_addr(&self) -> PhysAddr {
        PhysAddr::from(self.start())
    }

    fn virt_addr(&self) -> VirtAddr {
        VirtAddr::from(self.start())
    }

    fn zero(&self) {
        unsafe {
            core::ptr::write_bytes(self.start() as *mut u8, 0, self.page_count * FRAME_SIZE);
        }
    }

    fn as_pte_slice<'a>(&self) -> &'a [Rv64PTE] {
        unsafe { core::slice::from_raw_parts_mut(self.start() as *mut u8 as _, ENTRY_COUNT) }
    }

    fn as_pte_mut_slice<'a>(&self) -> &'a mut [Rv64PTE] {
        unsafe { core::slice::from_raw_parts_mut(self.start() as *mut u8 as _, ENTRY_COUNT) }
    }
}

/// Implement [PhysPage](ptable::PhysPage) for FrameTracker
impl PhysPage for FrameTracker {
    fn phys_addr(&self) -> PhysAddr {
        self.start_phy_addr()
    }

    fn as_slice(&self) -> &[u8] {
        self.deref()
    }

    fn as_mut_slice(&mut self) -> &mut [u8] {
        self.deref_mut()
    }
}
