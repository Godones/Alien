use crate::{DomainType, SharedHeapAllocator, TaskShimImpl};
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use config::{FRAME_BITS, FRAME_SIZE};
use core::ops::Range;
use core::sync::atomic::AtomicBool;
use fdt::Fdt;
use interface::*;
use ksync::Mutex;
use libsyscall::Syscall;
use log::{info, warn};
use platform::iprint;
use spin::Lazy;

static DOMAIN_PAGE_MAP: Lazy<Mutex<BTreeMap<u64, Vec<(usize, usize)>>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));

static DOMAIN_SYSCALL: Lazy<Mutex<BTreeMap<u64, usize>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));
static DOMAIN_SHARE_ALLOCATOR: Lazy<Mutex<BTreeMap<u64, usize>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));
static DOMAIN_TASKSHIM_IMPL: Lazy<Mutex<BTreeMap<u64, usize>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));

pub fn register_domain_syscall_resource(domain_id: u64, syscall_addr: usize) {
    DOMAIN_SYSCALL.lock().insert(domain_id, syscall_addr);
}

pub fn register_domain_heap_resource(domain_id: u64, heap_addr: usize) {
    DOMAIN_SHARE_ALLOCATOR.lock().insert(domain_id, heap_addr);
}

pub fn register_domain_taskshim_resource(domain_id: u64, taskshim_addr: usize) {
    DOMAIN_TASKSHIM_IMPL.lock().insert(domain_id, taskshim_addr);
}

pub struct DomainSyscall;

impl Syscall for DomainSyscall {
    fn sys_alloc_pages(&self, domain_id: u64, n: usize) -> *mut u8 {
        let n = n.next_power_of_two();
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

        {
            let mut binding = DOMAIN_TASKSHIM_IMPL.lock();
            let ptr = binding.remove(&domain_id).unwrap();
            let _taskshim = unsafe { Box::from_raw(ptr as *mut TaskShimImpl) };
            drop(_taskshim);
            warn!("[Domain: {}] free TaskShimImpl resource", domain_id);
        }
        unwind();
    }

    fn sys_read_timer(&self) -> u64 {
        timer::read_timer() as u64
    }

    fn check_kernel_space(&self, start: usize, size: usize) -> bool {
        mem::is_in_kernel_space(start, size)
    }

    fn sys_get_blk_domain(&self) -> Option<Arc<dyn interface::BlkDeviceDomain>> {
        crate::query_domain("blk").map(|blk| match blk {
            DomainType::BlkDeviceDomain(blk) => blk,
            _ => panic!("blk domain type error"),
        })
    }

    fn sys_get_shadow_blk_domain(&self) -> Option<Arc<dyn BlkDeviceDomain>> {
        crate::query_domain("shadow_blk").map(|blk| match blk {
            DomainType::BlkDeviceDomain(blk) => blk,
            _ => panic!("blk domain type error"),
        })
    }

    fn sys_get_uart_domain(&self) -> Option<Arc<dyn interface::UartDomain>> {
        crate::query_domain("uart").map(|uart| match uart {
            DomainType::UartDomain(uart) => uart,
            _ => panic!("uart domain type error"),
        })
    }

    fn sys_get_gpu_domain(&self) -> Option<Arc<dyn GpuDomain>> {
        crate::query_domain("gpu").map(|gpu| match gpu {
            DomainType::GpuDomain(gpu) => gpu,
            _ => panic!("gpu domain type error"),
        })
    }

    fn sys_get_input_domain(&self, ty: &str) -> Option<Arc<dyn InputDomain>> {
        crate::query_domain(ty).map(|input| match input {
            DomainType::InputDomain(input) => input,
            _ => panic!("input domain type error"),
        })
    }

    fn sys_get_rtc_domain(&self) -> Option<Arc<dyn RtcDomain>> {
        crate::query_domain("rtc").map(|rtc| match rtc {
            DomainType::RtcDomain(rtc) => rtc,
            _ => panic!("rtc domain type error"),
        })
    }
    fn sys_get_cache_blk_domain(&self) -> Option<Arc<dyn CacheBlkDeviceDomain>> {
        crate::query_domain("cache_blk").map(|cache_blk| match cache_blk {
            DomainType::CacheBlkDeviceDomain(cache_blk) => cache_blk,
            _ => panic!("cache_blk domain type error"),
        })
    }

    fn sys_get_devices_domain(&self) -> Option<Arc<dyn DevicesDomain>> {
        crate::query_domain("devices").map(|devices| match devices {
            DomainType::DevicesDomain(devices) => devices,
            _ => panic!("devices domain type error"),
        })
    }

    fn blk_crash_trick(&self) -> bool {
        BLK_CRASH.load(core::sync::atomic::Ordering::Relaxed)
    }

    fn sys_get_dtb(&self) -> &'static [u8] {
        let ptr = platform::platform_dtb_ptr();
        let fdt = unsafe { Fdt::from_ptr(ptr as *const u8) }.unwrap();
        let size = fdt.total_size();
        unsafe { core::slice::from_raw_parts(ptr as _, size) }
    }
}

static BLK_CRASH: AtomicBool = AtomicBool::new(true);

fn unwind() -> ! {
    BLK_CRASH.store(false, core::sync::atomic::Ordering::Relaxed);
    continuation::unwind()
}
