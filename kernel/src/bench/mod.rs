mod proxy_block;
mod unwind_block;

use alloc::{boxed::Box, sync::Arc};

use arch::read_cycle;
use basic::time::read_time_us;
use interface::{BlkDeviceDomain, DomainType};
use shared_heap::DVec;
use unwind_block::MemoryImg;

use crate::{
    bench::{proxy_block::BlkDomainProxy, unwind_block::UnwindMemoryImg},
    domain_helper::query_domain,
    domain_loader::loader::DomainLoader,
};

pub fn test_func_cycle() {
    test_no_domain();
    test_unwind_domain();
    test_in_domain();
    test_in_domain_fake();
}

const TEST_SIZE: usize = 1000;

#[inline(never)]
fn test_func(name: &str, func: impl Fn() -> ()) {
    let start = read_cycle();
    let start_us = read_time_us();
    for _ in 0..TEST_SIZE {
        func();
    }
    let end = read_cycle();
    let end_us = read_time_us();
    println!("Test: {} cost: {} cycles", name, end - start);
    println!("Test: {} cost: {} us", name, end_us - start_us);
}

fn test_read_block(blk_device: Arc<dyn BlkDeviceDomain>) {
    let mut blk = DVec::new_uninit(512);
    let start = read_cycle();
    let start_us = read_time_us();
    for i in 0..TEST_SIZE {
        blk = blk_device.read_block((i % 2) as u32, blk).unwrap();
    }
    let end = read_cycle();
    let end_us = read_time_us();
    println!("Test: read_block cost: {} cycles", end - start);
    println!("Test: read_block cost: {} us", end_us - start_us);
}

fn prepare(blk_device: Arc<dyn BlkDeviceDomain>) {
    let mut blk = DVec::new_uninit(512);
    for i in 0..TEST_SIZE {
        blk = blk_device.read_block((i % 2) as u32, blk).unwrap();
    }
    blk_device.flush().unwrap();
}

fn create_mem_img(disk: &[u8]) -> Arc<dyn BlkDeviceDomain> {
    let ptr_range = disk.as_ptr() as usize..(disk.as_ptr() as usize + disk.len());
    let mem_block = MemoryImg::new();
    mem_block.init(&ptr_range).unwrap();
    let mem_block: Arc<dyn BlkDeviceDomain> = Arc::new(mem_block);
    mem_block
}

fn create_unwind_mem_img(disk: &[u8]) -> Arc<dyn BlkDeviceDomain> {
    let ptr_range = disk.as_ptr() as usize..(disk.as_ptr() as usize + disk.len());
    let mem_block = MemoryImg::new();
    mem_block.init(&ptr_range).unwrap();
    Arc::new(UnwindMemoryImg::new(mem_block))
}

fn test_no_domain() {
    let disk = [0u8; 1024];
    println_color!(32, "Test no domain");
    let mem_block = create_mem_img(&disk);
    prepare(mem_block.clone());
    // call empty function
    test_func("empty function", || {
        mem_block.flush().unwrap();
    });
    test_read_block(mem_block);
    println_color!(32, "Test no domain end");
}

fn test_unwind_domain() {
    let disk = [0u8; 1024];
    println_color!(32, "Test unwind domain");
    let unwind_block = create_unwind_mem_img(&disk);
    prepare(unwind_block.clone());
    // call empty function
    test_func("empty function", || {
        unwind_block.flush().unwrap();
    });
    test_read_block(unwind_block);
    println_color!(32, "Test unwind domain end");
}

fn create_proxy_domain(disk: &[u8]) -> Arc<dyn BlkDeviceDomain> {
    let ptr_range = disk.as_ptr() as usize..(disk.as_ptr() as usize + disk.len());
    let mem_block = MemoryImg::new();
    mem_block.init(&ptr_range).unwrap();
    let domain_mem_block = BlkDomainProxy::new(
        Box::new(UnwindMemoryImg::new(mem_block)),
        DomainLoader::empty(),
    );
    let domain_mem_block = Arc::new(domain_mem_block);
    domain_mem_block
}

fn test_in_domain_fake() {
    let disk = [0u8; 1024];
    println_color!(32, "Test in domain fake");
    let domain_mem_block = create_proxy_domain(&disk);
    prepare(domain_mem_block.clone());
    // call empty function
    test_func("empty function", || {
        domain_mem_block.flush().unwrap();
    });
    test_read_block(domain_mem_block);
    println_color!(32, "Test in domain fake end");
}

fn test_in_domain() {
    println_color!(32, "Test in domain");
    let domain_mem_block = query_domain("bench_block").unwrap();
    let domain_mem_block = match domain_mem_block {
        DomainType::BlkDeviceDomain(domain) => domain,
        _ => panic!("Domain type error"),
    };
    prepare(domain_mem_block.clone());
    // call empty function
    test_func("empty function", || {
        domain_mem_block.flush().unwrap();
    });
    test_read_block(domain_mem_block);
    println_color!(32, "Test in domain end");
}
