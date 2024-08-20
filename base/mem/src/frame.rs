use alloc::boxed::Box;
use core::{
    ops::{Deref, DerefMut},
    sync::atomic::AtomicUsize,
};

use config::{FRAME_BITS, FRAME_SIZE};
use ksync::Mutex;
use log::trace;
use memory_addr::{PhysAddr, VirtAddr};
use page_table::{NotLeafPage, PagingIf, Rv64PTE, ENTRY_COUNT};
use pager::{PageAllocator, PageAllocatorExt};
use platform::println;
use ptable::PhysPage;

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
        .unwrap_or_else(|_| panic!("alloc {} frame failed", num));
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
        .unwrap_or_else(|_| panic!("free frame start:{:#x},num:{} failed", start, num))
}

#[derive(Debug)]
pub struct FrameTracker {
    start_page: usize,
    page_count: usize,
    dealloc: bool,
}

extern "C" {
    fn strampoline();
}

impl FrameTracker {
    pub fn new(start_page: usize, page_count: usize, dealloc: bool) -> Self {
        Self {
            start_page,
            page_count,
            dealloc,
        }
    }
    pub fn create_trampoline() -> Self {
        let trampoline_phy_addr = strampoline as usize;
        assert_eq!(trampoline_phy_addr % FRAME_SIZE, 0);
        Self {
            start_page: trampoline_phy_addr >> FRAME_BITS,
            page_count: 1,
            dealloc: false,
        }
    }

    pub fn start(&self) -> usize {
        self.start_page << FRAME_BITS
    }
}

impl PhysPage for FrameTracker {
    fn phys_addr(&self) -> PhysAddr {
        PhysAddr::from(self.start())
    }

    fn as_bytes(&self) -> &[u8] {
        self.deref()
    }

    fn as_mut_bytes(&mut self) -> &mut [u8] {
        self.deref_mut()
    }
    fn read_value_atomic(&self, offset: usize) -> usize {
        let ptr = self.start() + offset;
        unsafe {
            AtomicUsize::from_ptr(ptr as *mut usize).load(core::sync::atomic::Ordering::Relaxed)
        }
    }
    fn write_value_atomic(&mut self, offset: usize, value: usize) {
        let ptr = self.start() + offset;
        unsafe {
            AtomicUsize::from_ptr(ptr as *mut usize)
                .store(value, core::sync::atomic::Ordering::Relaxed)
        }
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        if self.dealloc {
            trace!("drop frame tracker: {:#x?}", self);
            FRAME_ALLOCATOR
                .lock()
                .free_pages(self.start_page, self.page_count)
                .unwrap_or_else(|_| {
                    panic!(
                        "free frame start:{:#x},num:{} failed",
                        self.start_page, self.page_count
                    )
                });
        }
    }
}

impl Deref for FrameTracker {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe {
            core::slice::from_raw_parts(self.start() as *const u8, FRAME_SIZE * self.page_count)
        }
    }
}
impl DerefMut for FrameTracker {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            core::slice::from_raw_parts_mut(self.start() as *mut u8, FRAME_SIZE * self.page_count)
        }
    }
}

pub fn alloc_frame_trackers(count: usize) -> FrameTracker {
    let frame = FRAME_ALLOCATOR
        .lock()
        .alloc_pages(count, FRAME_SIZE)
        .unwrap_or_else(|_| panic!("alloc {} frame failed", count));
    trace!("alloc frame [{}] start page: {:#x}", count, frame);
    FrameTracker::new(frame, count, true)
}

pub struct VmmPageAllocator;

impl NotLeafPage<Rv64PTE> for FrameTracker {
    fn phys_addr(&self) -> PhysAddr {
        PhysAddr::from(self.start_page << FRAME_BITS)
    }

    fn virt_addr(&self) -> VirtAddr {
        VirtAddr::from(self.start_page << FRAME_BITS)
    }

    fn zero(&self) {
        let ptr = self.start();
        unsafe {
            core::ptr::write_bytes(ptr as *mut u8, 0, self.page_count * FRAME_SIZE);
        }
    }

    fn as_pte_slice<'a>(&self) -> &'a [Rv64PTE] {
        let ptr = self.start();
        unsafe { core::slice::from_raw_parts(ptr as _, ENTRY_COUNT) }
    }

    fn as_pte_mut_slice<'a>(&self) -> &'a mut [Rv64PTE] {
        let ptr = self.start();
        unsafe { core::slice::from_raw_parts_mut(ptr as _, ENTRY_COUNT) }
    }
}

impl PagingIf<Rv64PTE> for VmmPageAllocator {
    fn alloc_frame() -> Option<Box<dyn NotLeafPage<Rv64PTE>>> {
        let frame = alloc_frame_trackers(1);
        Some(Box::new(frame))
    }
}
