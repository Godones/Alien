#![no_std]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spin::Lazy;
use config::FRAME_BITS;
use ksync::Mutex;
use libsyscall::Syscall;
use platform::println;

pub struct DomainSyscall;

impl Syscall for DomainSyscall{
    fn sys_alloc_pages(&self, domain_id: u64, n: usize) -> *mut u8 {
        println!("alloc pages: [domain: {}], pages: {}", domain_id, n);
        let page = mem::alloc_frames(n);
        let mut binding = DOMAIN_PAGE_MAP.lock();
        let vec = binding.entry(domain_id).or_insert(Vec::new());
        vec.push(page as usize >> FRAME_BITS);
        page
    }

    fn sys_free_pages(&self, domain_id: u64, p: *mut u8, n: usize) {
        println!("free pages: [domain: {}], ptr: {:p}, pages: {}", domain_id, p, n);
        let mut binding = DOMAIN_PAGE_MAP.lock();
        let vec = binding.entry(domain_id).or_insert(Vec::new());
        let start = p as usize >> FRAME_BITS;
        for i in 0..n {
            vec.retain(|&x| x != start + i)
        }
        mem::free_frames(p, n);
    }

    fn sys_write_console(&self, s: &str) {
        println!("{}", s);
    }
}




static DOMAIN_PAGE_MAP: Lazy<Mutex<BTreeMap<u64,Vec<usize>>>> = Lazy::new(|| Mutex::new(BTreeMap::new()));