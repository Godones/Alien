use alloc::{string::ToString, sync::Arc};
use core::{any::Any, sync::atomic::AtomicBool};

use config::FRAME_BITS;
use corelib::{domain_info::DomainDataInfo, CoreFunction};
use interface::*;
use platform::iprint;
use task_meta::{OperationResult, TaskOperation};

use crate::{
    domain_helper::{resource::DOMAIN_RESOURCE, DOMAIN_CREATE, DOMAIN_INFO},
    domain_proxy::*,
    error::{AlienError, AlienResult},
};

pub static DOMAIN_SYS: &'static dyn CoreFunction = &DomainSyscall;

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
        let mut info = DOMAIN_INFO.lock();
        info.domain_list
            .get_mut(&domain_id)
            .map(|d| d.panic_count += 1);
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

    fn sys_create_domain(
        &self,
        domain_file_name: &str,
        identifier: &mut [u8],
    ) -> AlienResult<DomainType> {
        DOMAIN_CREATE
            .get()
            .unwrap()
            .create_domain(domain_file_name, identifier)
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
        let old_domain_id = old_domain.as_ref().map(|d| d.domain_id());
        let (domain_info, new_domain_id) = match old_domain {
            Some(DomainType::GpuDomain(gpu)) => {
                let old_domain_id = gpu.domain_id();
                let (id, new_domain, loader) = crate::domain_loader::creator::create_domain(
                    ty,
                    new_domain_name,
                    None,
                    Some(old_domain_id),
                )
                .ok_or(AlienError::EINVAL)?;
                let gpu_proxy = gpu.downcast_arc::<GpuDomainProxy>().unwrap();
                let domain_info = loader.domain_file_info();
                gpu_proxy.replace(new_domain, loader)?;
                println!(
                    "Try to replace domain: {} with domain: {} ok",
                    old_domain_name, new_domain_name
                );
                Ok((domain_info, id))
            }
            Some(DomainType::ShadowBlockDomain(shadow_blk)) => {
                let old_domain_id = shadow_blk.domain_id();
                let (id, new_domain, loader) = crate::domain_loader::creator::create_domain(
                    ty,
                    new_domain_name,
                    None,
                    Some(old_domain_id),
                )
                .ok_or(AlienError::EINVAL)?;
                let shadow_blk_proxy = shadow_blk.downcast_arc::<ShadowBlockDomainProxy>().unwrap();
                let domain_info = loader.domain_file_info();
                shadow_blk_proxy.replace(new_domain, loader)?;
                println!(
                    "Try to replace domain: {} with domain: {} ok",
                    old_domain_name, new_domain_name
                );
                Ok((domain_info, id))
            }
            Some(DomainType::SchedulerDomain(scheduler)) => {
                let old_domain_id = scheduler.domain_id();
                let (id, new_domain, loader) = crate::domain_loader::creator::create_domain(
                    ty,
                    new_domain_name,
                    None,
                    Some(old_domain_id),
                )
                .ok_or(AlienError::EINVAL)?;
                let scheduler_proxy = scheduler.downcast_arc::<SchedulerDomainProxy>().unwrap();
                let domain_info = loader.domain_file_info();
                scheduler_proxy.replace(new_domain, loader)?;
                println!(
                    "Try to replace {:?} [{}] with [{}] ok",
                    ty, old_domain_name, new_domain_name
                );
                Ok((domain_info, id))
            }
            Some(DomainType::LogDomain(logger)) => {
                let old_domain_id = logger.domain_id();
                let (id, new_domain, loader) = crate::domain_loader::creator::create_domain(
                    ty,
                    new_domain_name,
                    None,
                    Some(old_domain_id),
                )
                .ok_or(AlienError::EINVAL)?;
                let logger_proxy = logger.downcast_arc::<LogDomainProxy>().unwrap();
                let domain_info = loader.domain_file_info();
                logger_proxy.replace(new_domain, loader)?;
                println!(
                    "Try to replace logger domain {} with {} ok",
                    old_domain_name, new_domain_name
                );
                Ok((domain_info, id))
            }

            Some(DomainType::InputDomain(input)) => {
                let old_domain_id = input.domain_id();
                let (id, new_domain, loader) = crate::domain_loader::creator::create_domain(
                    ty,
                    new_domain_name,
                    None,
                    Some(old_domain_id),
                )
                .ok_or(AlienError::EINVAL)?;
                let input_proxy = input.downcast_arc::<InputDomainProxy>().unwrap();
                let domain_info = loader.domain_file_info();
                input_proxy.replace(new_domain, loader)?;
                println!(
                    "Try to replace input domain {} with {} ok",
                    old_domain_name, new_domain_name
                );
                Ok((domain_info, id))
            }

            Some(DomainType::NetDeviceDomain(nic)) => {
                let old_domain_id = nic.domain_id();
                let (id, new_domain, loader) = crate::domain_loader::creator::create_domain(
                    ty,
                    new_domain_name,
                    None,
                    Some(old_domain_id),
                )
                .ok_or(AlienError::EINVAL)?;
                let nic_proxy = nic.downcast_arc::<NetDeviceDomainProxy>().unwrap();
                let domain_info = loader.domain_file_info();
                nic_proxy.replace(new_domain, loader)?;
                println!(
                    "Try to replace net device domain {} with {} ok",
                    old_domain_name, new_domain_name
                );
                Ok((domain_info, id))
            }

            Some(DomainType::VfsDomain(vfs)) => {
                let old_domain_id = vfs.domain_id();
                let (id, new_domain, loader) = crate::domain_loader::creator::create_domain(
                    ty,
                    new_domain_name,
                    None,
                    Some(old_domain_id),
                )
                .ok_or(AlienError::EINVAL)?;
                let vfs_proxy = vfs.downcast_arc::<VfsDomainProxy>().unwrap();
                let domain_info = loader.domain_file_info();
                vfs_proxy.replace(new_domain, loader)?;
                println!(
                    "Try to replace vfs domain {} with {} ok",
                    old_domain_name, new_domain_name
                );
                Ok((domain_info, id))
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
        }?;
        let domain_data = DomainDataInfo {
            name: old_domain_name.to_string(),
            ty,
            panic_count: 0,
            file_info: domain_info,
        };

        let mut info = DOMAIN_INFO.lock();
        info.domain_list.remove(&old_domain_id.unwrap());
        info.domain_list.insert(new_domain_id, domain_data);
        Ok(())
    }
    fn sys_reload_domain(&self, domain_name: &str) -> AlienResult<()> {
        let domain = super::query_domain(domain_name).ok_or(AlienError::EINVAL)?;
        match domain {
            DomainType::BlkDeviceDomain(blk) => {
                let old_domain_id = blk.domain_id();
                let blk_proxy = blk.downcast_arc::<BlkDomainProxy>().unwrap();
                let domain_loader = blk_proxy.domain_loader();
                let (_id, new_domain, loader) =
                    crate::domain_loader::creator::create_domain_with_loader(
                        domain_loader,
                        Some(old_domain_id),
                    )
                    .ok_or(AlienError::EINVAL)?;
                blk_proxy.reload(new_domain, loader)?;
                println!("Try to reload blk device domain ok");
                Ok(())
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
            TaskOperation::Create(task_meta) => {
                crate::task::add_one_task(task_meta, false).map(OperationResult::KstackTop)
            }
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
            TaskOperation::SetPriority(nice) => {
                crate::task::set_task_priority(nice);
                Ok(OperationResult::Null)
            }
            TaskOperation::GetPriority => {
                let nice = crate::task::get_task_priority();
                Ok(OperationResult::Priority(nice))
            }
        }
    }
    fn checkout_shared_data(&self) -> AlienResult<()> {
        crate::domain_helper::checkout_shared_data();
        Ok(())
    }

    fn domain_info(&self) -> AlienResult<Arc<dyn Any + Send + Sync>> {
        let info = DOMAIN_INFO.clone();
        Ok(info)
    }
}

extern "C" {
    fn strampoline();
}
static BLK_CRASH: AtomicBool = AtomicBool::new(true);
fn unwind() {
    BLK_CRASH.store(false, core::sync::atomic::Ordering::Relaxed);
}
