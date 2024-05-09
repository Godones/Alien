use core::alloc::GlobalAlloc;

use config::FRAME_SIZE;
use log::trace;
use talc::{ClaimOnOom, Talc, Talck};

use crate::{alloc_frames, free_frames, KERNEL_HEAP};

static HEAP_ALLOCATOR: Talck<ksync::Mutex<()>, ClaimOnOom> = Talc::new(unsafe {
    ClaimOnOom::new(talc::Span::from_const_array(core::ptr::addr_of!(
        KERNEL_HEAP
    )))
})
.lock();

pub struct TalcAllocator;

unsafe impl GlobalAlloc for TalcAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        if layout.size() >= 5 * 1024 * 1024 {
            let need_page = (layout.size() + FRAME_SIZE - 1) / FRAME_SIZE;
            trace!("alloc big page: {:#x}", layout.size());
            alloc_frames(need_page)
        } else {
            HEAP_ALLOCATOR.alloc(layout)
        }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        if layout.size() >= 5 * 1024 * 1024 {
            let need_page = (layout.size() + FRAME_SIZE - 1) / FRAME_SIZE;
            trace!("free big page: {:#x}", layout.size());
            free_frames(ptr, need_page);
        } else {
            HEAP_ALLOCATOR.dealloc(ptr, layout);
        }
    }
}
