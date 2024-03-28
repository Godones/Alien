use alloc::{boxed::Box, collections::BTreeMap, vec::Vec};
use core::sync::atomic::AtomicBool;

use config::FRAME_BITS;
use corelib::CoreFunction;
use interface::*;
use ksync::Mutex;
use log::{info, warn};
use platform::iprint;
use spin::Lazy;

use crate::SharedHeapAllocator;

static DOMAIN_PAGE_MAP: Lazy<Mutex<BTreeMap<u64, Vec<(usize, usize)>>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));

static DOMAIN_SYSCALL: Lazy<Mutex<BTreeMap<u64, usize>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));
static DOMAIN_SHARE_ALLOCATOR: Lazy<Mutex<BTreeMap<u64, usize>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));

pub fn register_domain_syscall_resource(domain_id: u64, syscall_addr: usize) {
    DOMAIN_SYSCALL.lock().insert(domain_id, syscall_addr);
}

pub fn register_domain_heap_resource(domain_id: u64, heap_addr: usize) {
    DOMAIN_SHARE_ALLOCATOR.lock().insert(domain_id, heap_addr);
}

pub struct DomainSyscall;

impl CoreFunction for DomainSyscall {
    fn sys_alloc_pages(&self, domain_id: u64, n: usize) -> *mut u8 {
        let n = n.next_power_of_two();
        let page = mem::alloc_frames(n);
        // info!(
        //     "[Domain: {}] alloc pages: {}, range:[{:#x}-{:#x}]",
        //     domain_id,
        //     n,
        //     page as usize,
        //     page as usize + n * FRAME_SIZE
        // );
        let mut binding = DOMAIN_PAGE_MAP.lock();
        let vec = binding.entry(domain_id).or_insert(Vec::new());
        vec.push((page as usize >> FRAME_BITS, n));
        page
    }

    fn sys_free_pages(&self, domain_id: u64, p: *mut u8, n: usize) {
        let n = n.next_power_of_two();
        info!("[Domain: {}] free pages: {}, ptr: {:p}", domain_id, n, p);
        let mut binding = DOMAIN_PAGE_MAP.lock();
        let vec = binding.entry(domain_id).or_insert(Vec::new());
        let start = p as usize >> FRAME_BITS;
        vec.retain(|(s, _)| *s != start);
        mem::free_frames(p, n);
    }

    fn sys_write_console(&self, s: &str) {
        iprint!("{}", s);
    }

    fn check_kernel_space(&self, start: usize, size: usize) -> bool {
        mem::is_in_kernel_space(start, size)
    }

    fn sys_backtrace(&self, domain_id: u64) {
        warn!("[Domain: {}] panic, resource should recycle.", domain_id);
        let mut binding = DOMAIN_PAGE_MAP.lock();
        if let Some(vec) = binding.remove(&domain_id) {
            for (page_start, n) in vec {
                let page_end = page_start + n;
                warn!(
                    "[Domain: {}] free pages: [{:#x}-{:#x}]",
                    domain_id,
                    page_start << FRAME_BITS,
                    page_end << FRAME_BITS
                );
                mem::free_frames((page_start << FRAME_BITS) as *mut u8, n);
            }
        }
        drop(binding); // release lock
        {
            let mut binding = DOMAIN_SYSCALL.lock();
            let ptr = binding.remove(&domain_id).unwrap();
            let _syscall_resource = unsafe { Box::from_raw(ptr as *mut DomainSyscall) };
            drop(_syscall_resource);
            warn!("[Domain: {}] free DomainSyscall resource", domain_id);
        }
        {
            let mut binding = DOMAIN_SHARE_ALLOCATOR.lock();
            let ptr = binding.remove(&domain_id).unwrap();
            let _allocator = unsafe { Box::from_raw(ptr as *mut SharedHeapAllocator) };
            drop(_allocator);
            warn!("[Domain: {}] free SharedHeapAllocator resource", domain_id);
        }

        unwind();
    }

    fn sys_switch_task(&self, now: *mut context::TaskContext, next: *const context::TaskContext) {
        kcore::task::switch(now, next)
    }

    fn sys_trampoline_addr(&self) -> usize {
        strampoline as usize
    }

    fn sys_kernel_satp(&self) -> usize {
        mem::kernel_satp()
    }

    fn sys_trap_from_user(&self) -> usize {
        kcore::trap::user_trap_vector as usize
    }

    fn sys_trap_to_user(&self) -> usize {
        kcore::trap::trap_return as usize
    }

    fn blk_crash_trick(&self) -> bool {
        BLK_CRASH.load(core::sync::atomic::Ordering::Relaxed)
    }

    fn sys_read_time_ms(&self) -> u64 {
        timer::get_time_ms() as u64
    }

    fn sys_get_domain(&self, name: &str) -> Option<DomainType> {
        crate::query_domain(name)
    }
}
extern "C" {
    fn strampoline();
}
static BLK_CRASH: AtomicBool = AtomicBool::new(true);

fn unwind() -> ! {
    BLK_CRASH.store(false, core::sync::atomic::Ordering::Relaxed);
    continuation::unwind()
}
