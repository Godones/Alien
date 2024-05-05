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
        // alloc doesn't have enough memory, and we just got called! we must free up some memory
        // we'll go through an example of how to handle this situation

        // we can inspect `layout` to estimate how much we should free up for this allocation
        // or we can extend by any amount (increasing powers of two has good time complexity)

        // this function will be repeatly called until we free up enough memory or
        // we return Err(()) causing allocation failure. Be careful to avoid conditions where
        // the arena isn't sufficiently extended indefinitely, causing an infinite loop

        // an arbitrary address limit for the sake of example

        let size = layout.size();
        let size = (size + FRAME_SIZE - 1) / FRAME_SIZE * FRAME_SIZE + 1; // more than 1 frame
        let alloc_heap = sbrk(size as isize) + size as isize;

        // println!("oom occur, alloc_heap: {:#x}", alloc_heap);
        // let old_arena = talc.get_arena();

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
