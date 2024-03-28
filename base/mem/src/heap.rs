use core::alloc::GlobalAlloc;

#[cfg(feature = "buddy")]
use buddy_system_allocator::LockedHeap;
use config::FRAME_SIZE;
use ksync::Mutex;
use log::trace;
#[cfg(feature = "rslab")]
use rslab::{init_slab_system, SlabAllocator};
#[cfg(feature = "talloc")]
use talc::{Talc, Talck};

use crate::frame::{alloc_frames, free_frames};

pub struct HeapAllocator {
    #[cfg(feature = "talloc")]
    allocator: Mutex<Talck>,
    #[cfg(feature = "buddy")]
    allocator: Mutex<LockedHeap<32>>,
    #[cfg(feature = "slab")]
    allocator: Mutex<SlabAllocator>,
}

unsafe impl GlobalAlloc for HeapAllocator {
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
            #[cfg(feature = "talloc")]
            allocator: Mutex::new(Talc::new().spin_lock()),
            #[cfg(feature = "buddy")]
            allocator: Mutex::new(LockedHeap::<32>::new()),
            #[cfg(feature = "slab")]
            allocator: Mutex::new(SlabAllocator),
        }
    }
    pub fn init(&self, heap: &mut [u8]) {
        #[cfg(feature = "talloc")]
        unsafe {
            self.allocator.lock().talc().init(heap.into())
        }
        #[cfg(feature = "buddy")]
        unsafe {
            self.allocator
                .lock()
                .lock()
                .init(heap.as_mut_ptr() as usize, heap.len())
        }
        #[cfg(feature = "slab")]
        unsafe {
            init_slab_system(FRAME_SIZE, 64);
        }
    }
}
