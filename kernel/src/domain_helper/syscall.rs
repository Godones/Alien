use alloc::{boxed::Box, collections::BTreeMap, vec::Vec};
use core::sync::atomic::AtomicBool;

use basic::task::TaskContext;
use config::FRAME_BITS;
use corelib::CoreFunction;
use interface::*;
use ksync::Mutex;
use log::warn;
use platform::iprint;
use spin::Lazy;

use crate::{
    domain_helper::{device::update_device_domain, SharedHeapAllocator, DOMAIN_CREATE},
    domain_proxy::{
        BlkDomainProxy, LogDomainProxy, ProxyExt, SchedulerDomainProxy, ShadowBlockDomainProxy,
    },
    error::{AlienError, AlienResult},
};

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
        debug!("[Domain: {}] free pages: {}, ptr: {:p}", domain_id, n, p);
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

        unwind();
    }
    fn sys_trampoline_addr(&self) -> usize {
        strampoline as usize
    }

    fn sys_kernel_satp(&self) -> usize {
        mem::kernel_satp()
    }

    fn sys_trap_from_user(&self) -> usize {
        crate::trap::user_trap_vector as usize
    }

    fn sys_trap_to_user(&self) -> usize {
        crate::trap::trap_return as usize
    }

    fn blk_crash_trick(&self) -> bool {
        BLK_CRASH.load(core::sync::atomic::Ordering::Relaxed)
    }

    fn sys_get_domain(&self, name: &str) -> Option<DomainType> {
        super::query_domain(name)
    }

    fn switch_task(&self, now: *mut TaskContext, next: *const TaskContext, next_tid: usize) {
        crate::domain_proxy::continuation::set_current_task_id(next_tid);
        crate::task::switch(now, next)
    }

    fn sys_create_domain(&self, identifier: &str) -> Option<DomainType> {
        DOMAIN_CREATE.get().unwrap().create_domain(identifier)
    }

    fn sys_register_domain(&self, ident: &str, ty: DomainTypeRaw, data: &[u8]) -> AlienResult<()> {
        crate::domain_loader::creator::register_domain_elf(ident, data.to_vec(), ty);
        Ok(())
    }

    fn sys_update_domain(
        &self,
        old_domain_name: &str,
        new_domain_name: &str,
        ty: DomainTypeRaw,
    ) -> AlienResult<()> {
        let old_domain = super::query_domain(old_domain_name);
        match old_domain {
            Some(DomainType::ShadowBlockDomain(shadow_blk)) => {
                let (_id, new_domain, loader) = crate::domain_loader::creator::create_domain(
                    DomainTypeRaw::ShadowBlockDomain,
                    new_domain_name,
                    None,
                )
                .ok_or(AlienError::EINVAL)?;
                let shadow_blk_proxy = shadow_blk.downcast_arc::<ShadowBlockDomainProxy>().unwrap();
                shadow_blk_proxy.replace(new_domain, loader)?;
                // todo!(release old domain's resource)
                warn!(
                    "Try to replace domain: {} with domain: {}",
                    old_domain_name, new_domain_name
                );
                Ok(())
            }
            Some(DomainType::SchedulerDomain(scheduler)) => {
                let (_id, new_domain, loader) = crate::domain_loader::creator::create_domain(
                    DomainTypeRaw::SchedulerDomain,
                    new_domain_name,
                    None,
                )
                .ok_or(AlienError::EINVAL)?;
                println!(
                    "Try to replace scheduler domain {} with {}",
                    old_domain_name, new_domain_name
                );
                let scheduler_proxy = scheduler.downcast_arc::<SchedulerDomainProxy>().unwrap();
                scheduler_proxy.replace(new_domain, loader)?;
                Err(AlienError::EINVAL)
            }
            Some(DomainType::LogDomain(logger)) => {
                let (_id, new_domain, loader) = crate::domain_loader::creator::create_domain(
                    DomainTypeRaw::LogDomain,
                    new_domain_name,
                    None,
                )
                .ok_or(AlienError::EINVAL)?;
                println!(
                    "Try to replace logger domain {} with {}",
                    old_domain_name, new_domain_name
                );
                let logger_proxy = logger.downcast_arc::<LogDomainProxy>().unwrap();
                logger_proxy.replace(new_domain, loader)?;
                Ok(())
            }

            None => {
                println!(
                    "<sys_update_domain> old domain {:?} not found",
                    old_domain_name
                );
                match ty {
                    DomainTypeRaw::GpuDomain => {
                        println!("update gpu domain: {}", new_domain_name);
                        update_device_domain(ty, new_domain_name)?;
                        Ok(())
                    }
                    _ => Err(AlienError::EINVAL),
                }
            }
            _ => {
                panic!("replace domain not support");
            }
        }
    }
    fn sys_reload_domain(&self, domain_name: &str) -> AlienResult<()> {
        let domain = super::query_domain(domain_name).ok_or(AlienError::EINVAL)?;
        match domain {
            DomainType::BlkDeviceDomain(blk) => {
                let blk_proxy = blk.downcast_arc::<BlkDomainProxy>().unwrap();
                blk_proxy.reload()
            }
            // todo!(release old domain's resource)
            ty => {
                panic!("reload domain {:?} not support", ty);
            }
        }
    }
    fn map_kstack_for_task(&self, task_id: usize, pages: usize) -> AlienResult<usize> {
        mem::map_kstack_for_task(task_id, pages)
    }
    fn unmapped_kstack_for_task(&self, task_id: usize, pages: usize) -> AlienResult<()> {
        mem::unmap_kstack_for_task(task_id, pages)
    }

    fn vaddr_to_paddr_in_kernel(&self, vaddr: usize) -> AlienResult<usize> {
        mem::query_kernel_space(vaddr).ok_or(AlienError::EINVAL)
    }
}

extern "C" {
    fn strampoline();
}
static BLK_CRASH: AtomicBool = AtomicBool::new(true);
fn unwind() -> ! {
    BLK_CRASH.store(false, core::sync::atomic::Ordering::Relaxed);
    crate::domain_proxy::continuation::unwind()
}
