use core::sync::atomic::AtomicBool;

use config::FRAME_BITS;
use corelib::CoreFunction;
use interface::*;
use log::warn;
use platform::iprint;
use task_meta::{OperationResult, TaskOperation};

use crate::{
    domain_helper::{resource::DOMAIN_RESOURCE, DOMAIN_CREATE},
    domain_proxy::*,
    error::{AlienError, AlienResult},
};

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
        DOMAIN_RESOURCE
            .lock()
            .insert_page_map(domain_id, (page as usize >> FRAME_BITS, n));
        page
    }

    fn sys_free_pages(&self, domain_id: u64, p: *mut u8, n: usize) {
        let n = n.next_power_of_two();
        debug!("[Domain: {}] free pages: {}, ptr: {:p}", domain_id, n, p);
        DOMAIN_RESOURCE
            .lock()
            .free_page_map(domain_id, p as usize >> FRAME_BITS);
        mem::free_frames(p, n);
    }

    fn sys_write_console(&self, s: &str) {
        iprint!("{}", s);
    }

    fn sys_backtrace(&self, domain_id: u64) {
        warn!("[Domain: {}] panic, resource should recycle.", domain_id);
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
        _ty: DomainTypeRaw,
    ) -> AlienResult<()> {
        let old_domain = super::query_domain(old_domain_name);
        match old_domain {
            Some(DomainType::GpuDomain(gpu)) => {
                let (id, new_domain, loader) = crate::domain_loader::creator::create_domain(
                    DomainTypeRaw::GpuDomain,
                    new_domain_name,
                    None,
                )
                .ok_or(AlienError::EINVAL)?;
                let gpu_proxy = gpu.downcast_arc::<GpuDomainProxy>().unwrap();
                gpu_proxy.replace(new_domain, loader, id)?;
                // todo!(release old domain's resource)
                println!(
                    "Try to replace domain: {} with domain: {} ok",
                    old_domain_name, new_domain_name
                );
                Ok(())
            }
            Some(DomainType::ShadowBlockDomain(shadow_blk)) => {
                let (id, new_domain, loader) = crate::domain_loader::creator::create_domain(
                    DomainTypeRaw::ShadowBlockDomain,
                    new_domain_name,
                    None,
                )
                .ok_or(AlienError::EINVAL)?;
                let shadow_blk_proxy = shadow_blk.downcast_arc::<ShadowBlockDomainProxy>().unwrap();
                shadow_blk_proxy.replace(new_domain, loader, id)?;
                // todo!(release old domain's resource)
                warn!(
                    "Try to replace domain: {} with domain: {} ok",
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
                    "Try to replace scheduler domain {} with {} ok",
                    old_domain_name, new_domain_name
                );
                let scheduler_proxy = scheduler.downcast_arc::<SchedulerDomainProxy>().unwrap();
                scheduler_proxy.replace(new_domain, loader)?;
                Err(AlienError::EINVAL)
            }
            Some(DomainType::LogDomain(logger)) => {
                let (id, new_domain, loader) = crate::domain_loader::creator::create_domain(
                    DomainTypeRaw::LogDomain,
                    new_domain_name,
                    None,
                )
                .ok_or(AlienError::EINVAL)?;
                println!(
                    "Try to replace logger domain {} with {} ok",
                    old_domain_name, new_domain_name
                );
                let logger_proxy = logger.downcast_arc::<LogDomainProxy>().unwrap();
                logger_proxy.replace(new_domain, loader, id)?;
                Ok(())
            }

            None => {
                println!(
                    "<sys_update_domain> old domain {:?} not found",
                    old_domain_name
                );
                Err(AlienError::EINVAL)
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
    fn vaddr_to_paddr_in_kernel(&self, vaddr: usize) -> AlienResult<usize> {
        mem::query_kernel_space(vaddr).ok_or(AlienError::EINVAL)
    }

    fn task_op(&self, op: TaskOperation) -> corelib::AlienResult<OperationResult> {
        match op {
            TaskOperation::Create(task_meta) => crate::task::add_one_task(task_meta, false)
                .map(|res| OperationResult::KstackTop(res)),
            TaskOperation::Wait => {
                crate::task::wait_now();
                Ok(OperationResult::Null)
            }
            TaskOperation::Wakeup(tid) => {
                crate::task::wake_up_wait_task(tid);
                Ok(OperationResult::Null)
            }
            TaskOperation::Yield => {
                crate::task::yield_now();
                Ok(OperationResult::Null)
            }
            TaskOperation::Exit => {
                crate::task::exit_now();
                Ok(OperationResult::Null)
            }
            TaskOperation::Remove(tid) => {
                crate::task::remove_task(tid);
                Ok(OperationResult::Null)
            }
            TaskOperation::Current => Ok(OperationResult::Current(crate::task::current_tid())),
            TaskOperation::ExitOver(tid) => {
                Ok(OperationResult::ExitOver(crate::task::is_task_exit(tid)))
            }
        }
    }
}

extern "C" {
    fn strampoline();
}
static BLK_CRASH: AtomicBool = AtomicBool::new(true);
fn unwind() -> ! {
    BLK_CRASH.store(false, core::sync::atomic::Ordering::Relaxed);
    crate::task::continuation::unwind()
}
