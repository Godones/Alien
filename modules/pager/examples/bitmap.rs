use std::alloc::{alloc, dealloc};
use std::ops::Range;

use pager::{Bitmap, PageAllocator};

fn main() {
    env_logger::init();
    let mut bitmap = Bitmap::<{ 4096 / 8 }>::new();
    let memory = unsafe { alloc(std::alloc::Layout::from_size_align(0x1000000, 0x1000).unwrap()) };
    let memory = memory as usize;
    let range = Range {
        start: memory,
        end: memory + 0x1000000,
    };
    bitmap.init(range).unwrap();
    let mut vec = vec![];
    for _ in 0..4096 {
        let page = bitmap.alloc(0).unwrap();
        vec.push(page);
    }

    println!("{:?}", bitmap); //all 1
    unsafe { dealloc(memory as *mut u8, std::alloc::Layout::from_size_align(0x1000000, 0x1000).unwrap()) }
}