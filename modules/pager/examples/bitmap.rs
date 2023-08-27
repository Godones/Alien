use std::alloc::{alloc, dealloc};
use std::ops::Range;

use pager::{Bitmap, PageAllocator};

fn specify_n_pages() {
    let mut bitmap = Bitmap::<{ 4096 / 8 }>::new();
    let memory = unsafe { alloc(std::alloc::Layout::from_size_align(0x1000_000, 0x1000).unwrap()) };
    let memory = memory as usize;
    let range = Range {
        start: memory,
        end: memory + 0x1000_000,
    };
    bitmap.init(range).unwrap();
    let mut vec = vec![];
    for _ in 0..4096 {
        let page = bitmap.alloc(0).unwrap();
        vec.push(page);
    }

    println!("{:?}", bitmap); //all 1
    unsafe {
        dealloc(
            memory as *mut u8,
            std::alloc::Layout::from_size_align(0x1000000, 0x1000).unwrap(),
        )
    }
}

fn not_specify_n_pages() {
    let mut bitmap = Bitmap::<0>::new();
    let memory = unsafe { alloc(std::alloc::Layout::from_size_align(0x1000_000, 0x1000).unwrap()) };
    let memory = memory as usize;
    let range = Range {
        start: memory,
        end: memory + 0x1000_000,
    }; // 4096 pages
       // because we not specify the number of pages, so the bitmap will use one page to store the bitmap
       // we only can use 4096 - 1 = 4095 pages
    bitmap.init(range).unwrap();
    let mut vec = vec![];
    for _ in 0..4095 {
        let page = bitmap.alloc(0).unwrap();
        vec.push(page);
    }
    println!("{:?}", bitmap); //all 1

    let (total, free) = bitmap.page_info();
    assert_eq!(total, 4095);
    assert_eq!(free, 0);

    unsafe {
        dealloc(
            memory as *mut u8,
            std::alloc::Layout::from_size_align(0x1000000, 0x1000).unwrap(),
        )
    }
}

fn main() {
    env_logger::init();
    println!("specify_n_pages: ");
    specify_n_pages();
    println!("not_specify_n_pages: ");
    not_specify_n_pages();
}
