use buddy_system_allocator::LockedHeap;
use config::FRAME_SIZE;
use log::trace;

use crate::{frame::alloc_frames, free_frames};

pub struct HeapAllocator {
    allocator: ksync::Mutex<LockedHeap<32>>,
}

unsafe impl core::alloc::GlobalAlloc for HeapAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        if layout.size() >= 5 * 1024 * 1024 {
            let need_page = (layout.size() + FRAME_SIZE - 1) / FRAME_SIZE;
            trace!("alloc big page: {:#x}", layout.size());
            alloc_frames(need_page)
        } else {
            self.allocator.lock().alloc(layout)
        }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        if layout.size() >= 5 * 1024 * 1024 {
            let need_page = (layout.size() + FRAME_SIZE - 1) / FRAME_SIZE;
            trace!("free big page: {:#x}", layout.size());
            free_frames(ptr, need_page);
        } else {
            self.allocator.lock().dealloc(ptr, layout);
        }
    }
}

impl HeapAllocator {
    pub const fn new() -> Self {
        Self {
            allocator: ksync::Mutex::new(LockedHeap::<32>::new()),
        }
    }
    pub fn init(&self, start: usize, size: usize) {
        unsafe { self.allocator.lock().lock().init(start, size) }
        println!("Kernel Heap size: {:#x}MB", size / 1024 / 1024);
    }
}
