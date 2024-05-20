use core::{
    alloc::{GlobalAlloc, Layout},
    sync::atomic::AtomicUsize,
};

use spin::Mutex;
use talc::{OomHandler, Span, Talc, Talck};

use crate::{common::FRAME_SIZE, memory::sbrk};

#[global_allocator]
static ALLOCATOR: Talck<Mutex<()>, MyOomHandler> = Talc::new(MyOomHandler).lock();

static HEAP_START: AtomicUsize = AtomicUsize::new(0);
static HEAP_END: AtomicUsize = AtomicUsize::new(0);
pub fn init_heap() {
    let heap_began = sbrk(0);
    let after_alloc = sbrk(0x1000) + 0x1000;
    // println!("init heap range: {:#x} - {:#x}", heap_began, after_alloc);
    HEAP_START.store(heap_began as usize, core::sync::atomic::Ordering::Relaxed);
    HEAP_END.store(after_alloc as usize, core::sync::atomic::Ordering::Relaxed);

    unsafe {
        ALLOCATOR
            .lock()
            .claim((Span::new(heap_began as *mut u8, after_alloc as *mut u8)));
    }
}

struct MyOomHandler;

impl OomHandler for MyOomHandler {
    fn handle_oom(talc: &mut Talc<Self>, layout: Layout) -> Result<(), ()> {
        let size = layout.size();
        let size = (size + FRAME_SIZE - 1) / FRAME_SIZE * FRAME_SIZE + 1; // more than 1 frame
        let alloc_heap = sbrk(size as isize) + size as isize;

        let old_end = HEAP_END.load(core::sync::atomic::Ordering::Relaxed);
        HEAP_END.store(alloc_heap as usize, core::sync::atomic::Ordering::Relaxed);
        let start = HEAP_START.load(core::sync::atomic::Ordering::Relaxed);
        let old_heap = Span::new(start as *mut u8, old_end as *mut u8);
        let new_heap = Span::new(start as *mut u8, alloc_heap as *mut u8);

        let res = unsafe { talc.extend(old_heap, new_heap) };
        if res == old_heap {
            // println!("oom failed");
            return Err(());
        }

        Ok(())
    }
}
