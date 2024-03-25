use alloc::format;
use config::{FRAME_BITS, FRAME_SIZE};
use core::mem::forget;
use core::ops::{Deref, DerefMut};
use ksync::Mutex;
use log::trace;
use memory_addr::{PhysAddr, VirtAddr};
use page_table::PagingIf;
use pager::{PageAllocator, PageAllocatorExt};
use platform::println;
use ptable::PhyPageMeta;

#[cfg(feature = "pager_bitmap")]
pub static FRAME_ALLOCATOR: Mutex<pager::Bitmap<0>> = Mutex::new(pager::Bitmap::new());
#[cfg(feature = "pager_buddy")]
pub static FRAME_ALLOCATOR: Mutex<pager::Zone<12>> = Mutex::new(pager::Zone::new());

pub fn init_frame_allocator(start: usize, end: usize) {
    let page_start = start / FRAME_SIZE;
    let page_end = end / FRAME_SIZE;
    let page_count = page_end - page_start;
    println!(
        "Page start:{:#x},end:{:#x},count:{:#x}",
        page_start, page_end, page_count
    );
    FRAME_ALLOCATOR
        .lock()
        .init(start..end)
        .expect("init frame allocator failed");
}

#[no_mangle]
pub fn alloc_frames(num: usize) -> *mut u8 {
    assert_eq!(num.next_power_of_two(), num);
    let start_page = FRAME_ALLOCATOR
        .lock()
        .alloc_pages(num, FRAME_SIZE)
        .expect(format!("alloc {} frame failed", num).as_str());
    let start_addr = start_page << FRAME_BITS;
    start_addr as *mut u8
}

#[no_mangle]
pub fn free_frames(addr: *mut u8, num: usize) {
    assert_eq!(num.next_power_of_two(), num);
    let start = addr as usize >> FRAME_BITS;
    FRAME_ALLOCATOR
        .lock()
        .free_pages(start, num)
        .expect(format!("free frame start:{:#x},num:{} failed", start, num).as_str());
}

#[derive(Debug)]
pub struct FrameTracker {
    // start page
    start: usize,
    // page count
    size: usize,
    // should be deallocated
    dealloc: bool,
}

impl FrameTracker {
    pub fn new(start_page: usize, size: usize, dealloc: bool) -> Self {
        Self {
            start: start_page,
            size,
            dealloc,
        }
    }
    pub fn from_addr(addr: usize, size: usize, dealloc: bool) -> Self {
        assert_eq!(addr % FRAME_SIZE, 0);
        Self::new(addr >> FRAME_BITS, size, dealloc)
    }

    pub fn end(&self) -> usize {
        self.start_addr() + FRAME_SIZE * self.size
    }
    pub fn as_ptr(&self) -> *mut u8 {
        self.start_addr() as *mut u8
    }
}

impl PhyPageMeta for FrameTracker {
    fn start_addr(&self) -> usize {
        self.start << FRAME_BITS
    }
    fn size(&self) -> usize {
        self.size * FRAME_SIZE
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        if self.dealloc {
            trace!("drop frame tracker: {:#x?}", self);
            FRAME_ALLOCATOR
                .lock()
                .free_pages(self.start, self.size)
                .expect(
                    format!(
                        "free frame start:{:#x},num:{} failed",
                        self.start, self.size
                    )
                    .as_str(),
                );
        }
    }
}

impl Deref for FrameTracker {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe {
            core::slice::from_raw_parts(self.start_addr() as *const u8, FRAME_SIZE * self.size)
        }
    }
}
impl DerefMut for FrameTracker {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            core::slice::from_raw_parts_mut(self.start_addr() as *mut u8, FRAME_SIZE * self.size)
        }
    }
}

pub fn alloc_frame_trackers(count: usize) -> FrameTracker {
    let frame = FRAME_ALLOCATOR
        .lock()
        .alloc_pages(count, FRAME_SIZE)
        .expect(format!("alloc {} frame failed", count).as_str());
    trace!("alloc frame [{}] start page: {:#x}", count, frame);
    FrameTracker::new(frame, count, true)
}

pub struct VmmPageAllocator;

impl PagingIf for VmmPageAllocator {
    fn alloc_frame() -> Option<PhysAddr> {
        let frame = alloc_frame_trackers(1);
        let start_addr = frame.start_addr();
        forget(frame);
        Some(PhysAddr::from(start_addr))
    }

    fn dealloc_frame(paddr: PhysAddr) {
        FrameTracker::from_addr(paddr.as_usize(), 1, true);
    }

    fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
        VirtAddr::from(paddr.as_usize())
    }
}
