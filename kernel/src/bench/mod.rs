mod mem_block;

use alloc::sync::Arc;

use arch::read_timer;
use interface::{BlkDeviceDomain, DomainType};
use mem_block::MemoryImg;
use rref::RRefVec;

use crate::domain_helper::query_domain;

pub fn test_func_cycle() {
    test_no_domain();
    test_in_domain();
}

const TEST_SIZE: usize = 1000;

#[inline(never)]
fn test_func(name: &str, func: impl Fn() -> ()) {
    let start = read_timer();
    for _ in 0..TEST_SIZE {
        func();
    }
    let end = read_timer();
    println!("Test: {} cost: {} cycles", name, end - start);
}

fn test_no_domain() {
    println_color!(32, "Test no domain");
    let disk = &[1u8; 1024];
    let ptr_range = disk.as_ptr() as usize..(disk.as_ptr() as usize + disk.len());
    let mem_block = MemoryImg::new();
    mem_block.init(&ptr_range).unwrap();
    let mem_block: Arc<dyn BlkDeviceDomain> = Arc::new(mem_block);
    // call empty function
    test_func("empty function", || {
        mem_block.flush().unwrap();
    });

    let start = read_timer();
    let mut blk = RRefVec::new_uninit(512);
    for _ in 0..TEST_SIZE {
        blk = mem_block.read_block(0, blk).unwrap();
    }
    let end = read_timer();
    println!("Test: read_block cost: {} cycles", end - start);
    println_color!(32, "Test no domain end");
}

fn test_in_domain() {
    println_color!(32, "Test in domain");
    let domain_mem_block = query_domain("bench_block").unwrap();
    let domain_mem_block = match domain_mem_block {
        DomainType::BlkDeviceDomain(domain) => domain,
        _ => panic!("Domain type error"),
    };
    // call empty function
    test_func("empty function", || {
        domain_mem_block.flush().unwrap();
    });

    let start = read_timer();
    let mut blk = RRefVec::new_uninit(512);
    for _ in 0..TEST_SIZE {
        blk = domain_mem_block.read_block(0, blk).unwrap();
    }
    let end = read_timer();
    println!("Test: read_block cost: {} cycles", end - start);

    println_color!(32, "Test in domain end");
}
