use std::alloc::{alloc, dealloc};
use std::ops::Range;

use buddy::{PageAllocator, Zone};

fn main() {
    env_logger::init();
    let mut zone = Zone::<12>::new();
    let memory = unsafe { alloc(std::alloc::Layout::from_size_align(0x1000000, 0x1000).unwrap()) };
    let memory = memory as usize;
    let range = Range {
        start: memory,
        end: memory + 0x1000000,
    };
    println!("{range:#x?}");

    zone.init(range).unwrap();
    show(&zone);
    let page1 = zone.alloc(11).unwrap();
    println!("get page1 id: {page1:#x?}");
    let page2 = zone.alloc(10).unwrap();
    println!("get page2 id: {page2:#x?}");
    show(&zone);

    let page3 = zone.alloc(10).unwrap();
    println!("get page3 id: {page3:#x?}");
    show(&zone);

    zone.free(page2, 10).unwrap();
    show(&zone);

    zone.free(page3, 10).unwrap();
    show(&zone); // page3 is merged to page2, order 11 will have 1 page

    zone.free(page1, 11).unwrap();
    show(&zone); // page1 is merged to page2, order 12 will have 1 page

    let mut vec = vec![];
    for _ in 0..4096 {
        let page = zone.alloc(0).unwrap();
        vec.push(page);
    }

    println!("alloc 4096 pages");

    unsafe { dealloc(memory as *mut u8, std::alloc::Layout::from_size_align(0x1000000, 0x1000).unwrap()) }
}


fn show(zone: &Zone<12>) {
    println!("{zone:#x?}");
}