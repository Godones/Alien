use core::{
    alloc::{Allocator, GlobalAlloc, Layout},
    ptr::NonNull,
};

use config::FRAME_SIZE;
use ksync::Mutex;
use log::trace;
use platform::config::HEAP_SIZE;
use spin::Lazy;
use talc::{ErrOnOom, Talc, Talck};

use crate::{alloc_frames, eheap, free_frames};

static HEAP_ALLOCATOR: Lazy<MyAllocator> = Lazy::new(|| MyAllocator::new());

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

pub struct MyAllocator(Talck<Mutex<()>, ErrOnOom>);

impl MyAllocator {
    fn new() -> Self {
        let talck = Talc::new(ErrOnOom).lock::<Mutex<()>>();
        unsafe {
            let heap = core::slice::from_raw_parts_mut(eheap as usize as *mut u8, HEAP_SIZE);
            let _res = talck.lock().claim(heap.as_mut().into()).unwrap();
        }

        Self(talck)
    }
}

unsafe impl GlobalAlloc for MyAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.0.allocate(layout).unwrap().as_mut_ptr();
        ptr
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0.deallocate(NonNull::new(ptr).unwrap(), layout);
    }
}
