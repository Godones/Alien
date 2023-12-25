use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use config::{FRAME_BITS, FRAME_SIZE};
use ksync::Mutex;
use libsyscall::Syscall;
use log::info;
use platform::{iprint, println};
use spin::Lazy;
static DOMAIN_PAGE_MAP: Lazy<Mutex<BTreeMap<u64, Vec<usize>>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));

pub struct DomainSyscall;

impl Syscall for DomainSyscall {
    fn sys_alloc_pages(&self, domain_id: u64, n: usize) -> *mut u8 {
        let page = mem::alloc_frames(n);
        info!(
            "[Domain: {}] alloc pages: {}, range:[{:#x}-{:#x}]",
            domain_id,
            n,
            page as usize,
            page as usize + n * FRAME_SIZE
        );
        let mut binding = DOMAIN_PAGE_MAP.lock();
        let vec = binding.entry(domain_id).or_insert(Vec::new());
        vec.push(page as usize >> FRAME_BITS);
        page
    }

    fn sys_free_pages(&self, domain_id: u64, p: *mut u8, n: usize) {
        info!("[Domain: {}] free pages: {}, ptr: {:p}", domain_id, n, p);
        let mut binding = DOMAIN_PAGE_MAP.lock();
        let vec = binding.entry(domain_id).or_insert(Vec::new());
        let start = p as usize >> FRAME_BITS;
        for i in 0..n {
            vec.retain(|&x| x != start + i)
        }
        mem::free_frames(p, n);
    }

    fn sys_write_console(&self, s: &str) {
        iprint!("{}", s);
    }

    fn backtrace(&self, domain_id: u64) {
        println!("[Domain: {}] backtrace:", domain_id);
        platform::system_shutdown();
    }
}
