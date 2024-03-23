use core::alloc::{GlobalAlloc, Layout};

use spin::Mutex;
use talc::{OomHandler, Span, Talc, Talck};

use crate::common::FRAME_SIZE;
use crate::memory::sbrk;

#[global_allocator]
static ALLOCATOR: Talck<Mutex<()>, MyOomHandler> = Talc::new(MyOomHandler).lock();

pub fn init_heap() {
    let heap_began = sbrk(0);
    let after_alloc = sbrk(0x1000) + 0x1000;
    // println!("init heap range: {:#x} - {:#x}", heap_began, after_alloc);
    unsafe {
        ALLOCATOR
            .talc()
            .init(Span::new(heap_began as *mut u8, after_alloc as *mut u8));
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
        let old_arena = talc.get_arena();

        // we're going to extend the arena upward, doubling its size
        // but we'll be sure not to extend past the limit
        let new_arena = old_arena
            .extend(0, old_arena.size())
            .below(alloc_heap as *mut u8);

        if new_arena == old_arena {
            // we won't be extending the arena, so we should return Err
            return Err(());
        }

        unsafe {
            // we're assuming the new memory up to ARENA_TOP_LIMIT is allocatable
            talc.extend(new_arena);
        };

        Ok(())
    }
}
