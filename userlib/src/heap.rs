use core::alloc::{GlobalAlloc, Layout};
use core::ptr::NonNull;

use buddy_system_allocator::LockedHeap;

use crate::common::FRAME_SIZE;
use crate::memory::sbrk;

#[global_allocator]
pub static HEAP: BuddyAllocator = BuddyAllocator::new();

// static mut HEAP_BUFFER: [u8; 1024 * 1024 * 16] = [0; 1024 * 1024 * 16];

pub fn init_heap() {
    // unsafe {
    //     HEAP.heap
    //         .lock()
    //         .init(HEAP_BUFFER.as_ptr() as usize, HEAP_BUFFER.len());
    // }
}

pub struct BuddyAllocator {
    heap: LockedHeap<32>,
}

impl BuddyAllocator {
    pub const fn new() -> Self {
        Self {
            heap: LockedHeap::empty(),
        }
    }
}

unsafe impl GlobalAlloc for BuddyAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let res = self.heap.lock().alloc(layout);
        let res = if res.is_err() {
            let size = layout.size();
            // align size to FRAME_SIZE
            let size = (size + FRAME_SIZE - 1) / FRAME_SIZE * FRAME_SIZE * 2;
            let res = sbrk(size as isize);
            if res == -1 {
                panic!("can't alloc, oom");
            }
            let res = res as usize;
            self.heap.lock().add_to_heap(res, res + size);
            // reallocate
            self.heap.lock().alloc(layout)
        } else {
            res
        };
        match res {
            Ok(p) => p.as_ptr(),
            Err(_) => {
                panic!("oom");
            }
        }
        // self.heap.lock().alloc(layout).unwrap().as_ptr()
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.heap.lock().dealloc(NonNull::new(ptr).unwrap(), layout);
    }
}
