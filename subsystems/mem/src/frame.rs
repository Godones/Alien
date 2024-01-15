use alloc::format;
use core::mem::forget;
use core::ops::{Deref, DerefMut};
use log::trace;
use page_table::addr::{PhysAddr, VirtAddr};
use config::{FRAME_BITS, FRAME_SIZE};
use ksync::Mutex;
use page_table::table::PagingIf;
use pager::{PageAllocator, PageAllocatorExt};
use platform::println;
use crate::manager::FRAME_REF_MANAGER;

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
    start: usize,
    size: usize,
}

impl FrameTracker {
    pub fn new(start: usize, size:usize) -> Self {
        Self { start ,size}
    }
    pub fn from_addr(addr: usize,size:usize) -> Self {
        assert_eq!(addr % FRAME_SIZE, 0);
        Self::new(addr >> FRAME_BITS,size)
    }
    pub fn start(&self) -> usize {
        self.start << FRAME_BITS
    }
    pub fn end(&self) -> usize {
        self.start() + FRAME_SIZE * self.size
    }
    pub fn as_ptr(&self) -> *mut u8 {
        self.start() as *mut u8
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        trace!("drop frame tracker: {:#x?}", self);
        let mut manager = FRAME_REF_MANAGER.lock();
        for i in 0..self.size {
            let _id = manager.dec_ref(self.start + i);
        }
    }
}

impl Deref for FrameTracker {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { core::slice::from_raw_parts(self.start() as *const u8, FRAME_SIZE*self.size) }
    }
}
impl DerefMut for FrameTracker {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::slice::from_raw_parts_mut(self.start() as *mut u8, FRAME_SIZE*self.size) }
    }
}


pub fn alloc_frame_trackers(count: usize) -> FrameTracker {
    let frame = FRAME_ALLOCATOR.lock().alloc_pages(count, FRAME_SIZE)
        .expect(format!("alloc {} frame failed", count).as_str());
    trace!("alloc frame [{}] start page: {:#x}", count, frame);
    for i in 0..count {
        let refs = FRAME_REF_MANAGER.lock().add_ref(frame + i);
        assert_eq!(refs, 1)
    }
    FrameTracker::new(frame,count)
}


pub struct VmmPageAllocator;

impl PagingIf for VmmPageAllocator {
    fn alloc_frame() -> Option<PhysAddr> {
        let frame = alloc_frame_trackers(1);
        let start_addr = frame.start();
        forget(frame);
        Some(PhysAddr::from(start_addr))
    }

    fn dealloc_frame(paddr: PhysAddr) {
        FrameTracker::from_addr(paddr.as_usize(),1);
    }

    fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
        VirtAddr::from(paddr.as_usize())
    }

    fn alloc_contiguous_frames(size: usize) -> Option<PhysAddr> {
        let frames = alloc_frame_trackers(size);
        let start_addr = frames.start();
        forget(frames);
        Some(PhysAddr::from(start_addr))
    }
}
