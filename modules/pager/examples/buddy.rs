use std::alloc::{alloc, dealloc};
use std::ops::Range;

use pager::{PageAllocator, PageAllocatorExt, Zone};

fn simple_alloc_dealloc() {
    let mut zone = Zone::<12>::new();
    let memory = unsafe { alloc(std::alloc::Layout::from_size_align(0x1000_000, 0x1000).unwrap()) };
    let memory = memory as usize;
    let range = Range {
        start: memory,
        end: memory + 0x1000_000,
    };
    println!("{range:#x?}");

    zone.init(range).unwrap();
    show(&zone);
    let page1 = zone.alloc(11).unwrap();
    println!("get page1 id: {page1:#x?}");
    let page2 = zone.alloc(10).unwrap();
    println!("get page2 id: {page2:#x?}");
    show(&zone);
    //
    let page3 = zone.alloc(10).unwrap();
    println!("get page3 id: {page3:#x?}");
    show(&zone);
    //
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
    show(&zone);
    vec.iter().for_each(|x| {
        zone.free(*x, 0).unwrap();
    });
    show(&zone);
    vec.clear();

    let mut count = 0;
    let mut map = Vec::new();
    loop {
        let rand_pages = rand::random::<usize>() % 128 + 1;
        let page = zone.alloc_pages(rand_pages, 0x1000);
        if page.is_err() {
            break;
        }
        let page = page.unwrap();
        map.push((page, rand_pages));
        count += rand_pages;
    }
    println!("alloc {} pages", count);
    show(&zone);
    map.iter().for_each(|(page, pages)| {
        zone.free_pages(*page, *pages).unwrap();
    });
    show(&zone);

    let page = zone.alloc_pages(1025, 0x1000).unwrap();
    println!("alloc 1025 pages, get page id: {:#x?}", page);
    let page = zone.alloc_pages(1023, 0x1000).unwrap();
    println!("alloc 1023 pages, get page id: {:#x?}", page);
    show(&zone);
    unsafe {
        dealloc(
            memory as *mut u8,
            std::alloc::Layout::from_size_align(0x1000_000, 0x1000).unwrap(),
        )
    }
}

fn alloc_dealloc() {
    let mut zone = Zone::<12>::new();
    let memory = unsafe {
        alloc(std::alloc::Layout::from_size_align(0x1000_000 + 0x2000 + 0x1000, 0x1000).unwrap())
    };
    let memory = memory as usize;
    let range = Range {
        start: memory,
        end: memory + 0x1000_000 + 0x1000 + 0x2000,
    };
    println!("{range:#x?}");
    zone.init(range).unwrap();
    show(&zone);

    let page1 = zone.alloc_pages(1, 0x1000).unwrap();
    println!("alloc 1 pages, get page id: {:#x?}", page1);
    let page2 = zone.alloc_pages(1, 0x1000).unwrap();
    println!("alloc 1 pages, get page id: {:#x?}", page2);
    let page3 = zone.alloc_pages(1, 0x1000).unwrap();
    println!("alloc 1 pages, get page id: {:#x?}", page3);
    show(&zone);
    zone.free(page1, 0).unwrap();
    zone.free(page3, 0).unwrap();
    show(&zone);
    zone.free_pages(page2, 1).unwrap();
    show(&zone);
}

fn main() {
    env_logger::init();
    alloc_dealloc();
}

fn show(zone: &Zone<12>) {
    println!("{zone:#x?}");
}
