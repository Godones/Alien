use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};

use lazy_static::lazy_static;

use kernel_sync::Mutex;
use pager::{Bitmap, PageAllocator, PageAllocatorExt};

use crate::config::{FRAME_BITS, FRAME_SIZE};

use super::manager::FrameRefManager;

lazy_static! {
    pub static ref FRAME_ALLOCATOR: Mutex<Bitmap<8192>> = Mutex::new(Bitmap::new());
}

lazy_static! {
    pub static ref FRAME_REF_MANAGER: Mutex<FrameRefManager> = Mutex::new(FrameRefManager::new());
}

extern "C" {
    fn ekernel();
}

pub fn init_frame_allocator(memory_end: usize) {
    let start = ekernel as usize;
    let end = memory_end;
    println!("memory start:{:#x},end:{:#x}", start, end);
    let page_start = start / FRAME_SIZE;
    let page_end = end / FRAME_SIZE;
    let page_count = page_end - page_start;
    println!(
        "page start:{:#x},end:{:#x},count:{:#x}",
        page_start, page_end, page_count
    );
    FRAME_ALLOCATOR.lock().init(start..end).unwrap();
}

#[derive(Debug)]
pub struct FrameTracker {
    id: usize,
}

pub fn addr_to_frame(addr: usize) -> FrameTracker {
    assert_eq!(addr % FRAME_SIZE, 0);
    FrameTracker::new(addr >> FRAME_BITS)
}

impl FrameTracker {
    pub fn new(id: usize) -> Self {
        Self { id }
    }
    #[allow(unused)]
    pub fn start(&self) -> usize {
        self.id << FRAME_BITS
    }
    #[allow(unused)]
    pub fn end(&self) -> usize {
        self.start() + FRAME_SIZE
    }
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn zero_init(&self) {
        let start_addr = self.start();
        unsafe {
            core::ptr::write_bytes(start_addr as *mut u8, 0, FRAME_SIZE);
        }
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        trace!("drop frame:{}", self.id);
        let _id = FRAME_REF_MANAGER.lock().dec_ref(self.id);
    }
}

impl Deref for FrameTracker {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { core::slice::from_raw_parts(self.start() as *const u8, FRAME_SIZE) }
    }
}

impl DerefMut for FrameTracker {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::slice::from_raw_parts_mut(self.start() as *mut u8, FRAME_SIZE) }
    }
}

/// 提供给slab分配器的接口
/// 这些页面需要保持连续
#[no_mangle]
pub fn alloc_frames(num: usize) -> *mut u8 {
    assert_eq!(num.count_ones(), 1);
    let start_page = FRAME_ALLOCATOR.lock().alloc_pages(num);
    if start_page.is_err() {
        panic!("alloc {} frame failed", num);
    }
    let start_page = start_page.unwrap();
    let start_addr = start_page << FRAME_BITS;
    trace!("slab alloc frame {} start:{:#x}", num, start_addr);
    start_addr as *mut u8
}

/// 提供给slab分配器的接口
#[no_mangle]
pub fn free_frames(addr: *mut u8, num: usize) {
    let start = addr as usize >> FRAME_BITS;
    trace!("slab free frame {} start:{:#x}", num, addr as usize);
    // make sure the num is 2^n
    assert_eq!(num.count_ones(), 1);
    FRAME_ALLOCATOR.lock().free_pages(start, num).unwrap();
}

pub fn frame_alloc() -> Option<FrameTracker> {
    let frame = FRAME_ALLOCATOR.lock().alloc(0);
    if frame.is_err() {
        return None;
    }
    let frame = frame.unwrap();
    FRAME_REF_MANAGER.lock().add_ref(frame);
    Some(FrameTracker::new(frame))
}

pub fn frames_alloc(count: usize) -> Option<Vec<FrameTracker>> {
    let mut ans = Vec::new();
    for _ in 0..count {
        let id = FRAME_ALLOCATOR.lock().alloc(0);
        if id.is_err() {
            return None;
        }
        let id = id.unwrap();
        FRAME_REF_MANAGER.lock().add_ref(id);
        ans.push(FrameTracker::new(id));
    }
    Some(ans)
}

pub fn frame_alloc_contiguous(count: usize) -> *mut u8 {
    let frame = FRAME_ALLOCATOR.lock().alloc_pages(count);
    if frame.is_err() {
        panic!("alloc {} frame failed, oom", count);
    }
    let frame = frame.unwrap();
    error!("alloc frame {} start:{:#x}", count, frame);
    for i in 0..count {
        let refs = FRAME_REF_MANAGER.lock().add_ref(frame + i);
        assert_eq!(refs, 1)
    }
    (frame << FRAME_BITS as u64) as *mut u8
}
