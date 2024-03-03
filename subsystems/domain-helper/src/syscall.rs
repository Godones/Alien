use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use config::{FRAME_BITS, FRAME_SIZE};
use core::arch::global_asm;
use interface::{CacheBlkDeviceDomain, FsDomain, GpuDomain, InputDomain, RtcDomain};
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
        println!("[Domain: {}] panic, resource should recycle.", domain_id);
        let mut binding = DOMAIN_PAGE_MAP.lock();
        if let Some(vec) = binding.remove(&domain_id) {
            println!("[Domain: {}] free {:?} pages", domain_id, vec.len());
            for page in vec {
                mem::free_frames((page << FRAME_BITS) as *mut u8, 1);
            }
        }
        drop(binding); // release lock
        unwind();
    }

    fn read_timer(&self) -> u64 {
        timer::read_timer() as u64
    }

    fn check_kernel_space(&self, start: usize, size: usize) -> bool {
        mem::is_in_kernel_space(start, size)
    }

    fn sys_get_blk_domain(&self) -> Option<Arc<dyn interface::BlkDeviceDomain>> {
        crate::query_domain("blk").map(|blk| unsafe { core::mem::transmute(blk) })
    }

    fn sys_get_uart_domain(&self) -> Option<Arc<dyn interface::UartDomain>> {
        crate::query_domain("uart").map(|uart| unsafe { core::mem::transmute(uart) })
    }

    fn sys_get_gpu_domain(&self) -> Option<Arc<dyn GpuDomain>> {
        crate::query_domain("gpu").map(|gpu| unsafe { core::mem::transmute(gpu) })
    }

    fn sys_get_input_domain(&self, ty: &str) -> Option<Arc<dyn InputDomain>> {
        crate::query_domain(ty).map(|input| unsafe { core::mem::transmute(input) })
    }

    fn sys_get_fs_domain(&self, ty: &str) -> Option<Arc<dyn FsDomain>> {
        crate::query_domain(ty).map(|fs| unsafe { core::mem::transmute(fs) })
    }

    fn sys_get_rtc_domain(&self) -> Option<Arc<dyn RtcDomain>> {
        crate::query_domain("rtc").map(|rtc| unsafe { core::mem::transmute(rtc) })
    }
    fn sys_get_cache_blk_domain(&self) -> Option<Arc<dyn CacheBlkDeviceDomain>> {
        crate::query_domain("cache_blk").map(|cache_blk| unsafe { core::mem::transmute(cache_blk) })
    }
}

fn unwind() -> ! {
    let continuation = proxy::pop_continuation().unwrap();
    println!("unwind, continuation: {:#x?}", &continuation);
    unsafe { __unwind(&continuation) }
}

extern "C" {
    fn __unwind(continuation: &continuation::Continuation) -> !;
}

global_asm!(
    r#"
    .section .text
    .global __unwind
    .type __unwind, @function
__unwind:
    ld x1, 1*8(a0)
    ld x2, 2*8(a0)
    ld x3, 3*8(a0)
    ld x4, 4*8(a0)
    ld x5, 5*8(a0)
    ld x6, 6*8(a0)
    ld x7, 7*8(a0)
    ld x8, 8*8(a0)
    ld x9, 9*8(a0)
    # ld x10, 10*8(a0)
    ld x11, 11*8(a0)
    ld x12, 12*8(a0)
    ld x13, 13*8(a0)
    ld x14, 14*8(a0)
    ld x15, 15*8(a0)
    ld x16, 16*8(a0)
    ld x17, 17*8(a0)
    ld x18, 18*8(a0)
    ld x19, 19*8(a0)
    ld x20, 20*8(a0)
    ld x21, 21*8(a0)
    ld x22, 22*8(a0)
    ld x23, 23*8(a0)
    ld x24, 24*8(a0)
    ld x25, 25*8(a0)
    ld x26, 26*8(a0)
    ld x27, 27*8(a0)
    ld x28, 28*8(a0)
    ld x29, 29*8(a0)
    ld x30, 30*8(a0)
    ld x31, 31*8(a0)
    
    mv gp, a0
    ld a0, 10*8(gp)  # a0==x10
    ld gp, 32*8(gp)  # gp -> func
    jr gp
   
    "#
);
