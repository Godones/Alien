#![no_std]
extern crate alloc;

use constants::AlienResult;
#[cfg(feature = "core_impl")]
pub use core_impl::*;
use interface::{DomainType, DomainTypeRaw};
use task_meta::TaskContext;

pub trait CoreFunction: Send + Sync {
    fn sys_alloc_pages(&self, domain_id: u64, n: usize) -> *mut u8;
    fn sys_free_pages(&self, domain_id: u64, p: *mut u8, n: usize);
    fn sys_write_console(&self, s: &str);
    fn sys_backtrace(&self, domain_id: u64);
    fn sys_trampoline_addr(&self) -> usize;
    fn sys_kernel_satp(&self) -> usize;
    fn sys_trap_from_user(&self) -> usize;
    fn sys_trap_to_user(&self) -> usize;
    /// This func will be deleted
    fn blk_crash_trick(&self) -> bool;
    fn sys_read_time_ms(&self) -> u64;
    fn sys_get_domain(&self, name: &str) -> Option<DomainType>;
    fn switch_task(&self, now: *mut TaskContext, next: *const TaskContext, next_tid: usize);
    fn sys_create_domain(&self, identifier: &str) -> Option<DomainType>;
    /// Register a new domain with the given name and type
    fn sys_register_domain(&self, ident: &str, ty: DomainTypeRaw, data: &[u8]) -> AlienResult<()>;
    /// Replace the old domain with the new domain
    fn sys_update_domain(
        &self,
        old_domain_name: &str,
        new_domain_name: &str,
        ty: DomainTypeRaw,
    ) -> AlienResult<()>;
    fn sys_reload_domain(&self, domain_name: &str) -> AlienResult<()>;
}

#[cfg(feature = "core_impl")]
mod core_impl {
    use alloc::boxed::Box;

    use constants::AlienResult;
    use interface::{DomainType, DomainTypeRaw};
    use spin::Once;
    use task_meta::TaskContext;

    use crate::CoreFunction;

    static CORE_FUNC: Once<Box<dyn CoreFunction>> = Once::new();

    extern "C" {
        fn sbss();
        fn ebss();
    }

    /// 清空.bss段
    fn clear_bss() {
        unsafe {
            core::slice::from_raw_parts_mut(
                sbss as usize as *mut u8,
                ebss as usize - sbss as usize,
            )
            .fill(0);
        }
    }

    pub fn init(syscall: Box<dyn CoreFunction>) {
        clear_bss();
        CORE_FUNC.call_once(|| syscall);
    }

    pub fn alloc_raw_pages(n: usize, domain_id: u64) -> *mut u8 {
        unsafe { CORE_FUNC.get_unchecked().sys_alloc_pages(domain_id, n) }
    }

    pub fn free_raw_pages(p: *mut u8, n: usize, domain_id: u64) {
        unsafe {
            CORE_FUNC.get_unchecked().sys_free_pages(domain_id, p, n);
        }
    }

    pub fn write_console(s: &str) {
        unsafe {
            CORE_FUNC.get_unchecked().sys_write_console(s);
        }
    }

    pub fn backtrace(domain_id: u64) {
        unsafe {
            CORE_FUNC.get_unchecked().sys_backtrace(domain_id);
        }
    }

    pub fn trampoline_addr() -> usize {
        static TRAMPOLINE_ADDR: Once<usize> = Once::new();
        unsafe {
            TRAMPOLINE_ADDR.call_once(|| CORE_FUNC.get_unchecked().sys_trampoline_addr());
            *TRAMPOLINE_ADDR.get_unchecked()
        }
    }

    pub fn kernel_satp() -> usize {
        static SATP: Once<usize> = Once::new();
        unsafe {
            SATP.call_once(|| CORE_FUNC.get_unchecked().sys_kernel_satp());
            *SATP.get_unchecked()
        }
    }

    pub fn trap_from_user() -> usize {
        static TRAP_FROM_USER: Once<usize> = Once::new();
        unsafe {
            TRAP_FROM_USER.call_once(|| CORE_FUNC.get_unchecked().sys_trap_from_user());
            *TRAP_FROM_USER.get_unchecked()
        }
    }

    pub fn trap_to_user() -> usize {
        static TRAP_TO_USER: Once<usize> = Once::new();
        unsafe {
            TRAP_TO_USER.call_once(|| CORE_FUNC.get_unchecked().sys_trap_to_user());
            *TRAP_TO_USER.get_unchecked()
        }
    }

    // todo!(delete)
    pub fn blk_crash_trick() -> bool {
        unsafe { CORE_FUNC.get_unchecked().blk_crash_trick() }
    }

    pub fn read_time_ms() -> u64 {
        unsafe { CORE_FUNC.get_unchecked().sys_read_time_ms() }
    }

    pub fn get_domain(name: &str) -> Option<DomainType> {
        unsafe { CORE_FUNC.get_unchecked().sys_get_domain(name) }
    }

    pub fn create_domain(identifier: &str) -> Option<DomainType> {
        unsafe { CORE_FUNC.get_unchecked().sys_create_domain(identifier) }
    }

    pub fn switch_task(now: *mut TaskContext, next: *const TaskContext, next_tid: usize) {
        unsafe {
            CORE_FUNC.get_unchecked().switch_task(now, next, next_tid);
        }
    }

    pub fn register_domain(ident: &str, ty: DomainTypeRaw, data: &[u8]) -> AlienResult<()> {
        unsafe {
            CORE_FUNC
                .get_unchecked()
                .sys_register_domain(ident, ty, data)
        }
    }

    pub fn update_domain(
        old_domain_name: &str,
        new_domain_name: &str,
        ty: DomainTypeRaw,
    ) -> AlienResult<()> {
        unsafe {
            CORE_FUNC
                .get_unchecked()
                .sys_update_domain(old_domain_name, new_domain_name, ty)
        }
    }

    pub fn reload_domain(domain_name: &str) -> AlienResult<()> {
        unsafe { CORE_FUNC.get_unchecked().sys_reload_domain(domain_name) }
    }
}
