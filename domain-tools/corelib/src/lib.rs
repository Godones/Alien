#![no_std]
extern crate alloc;

use context::TaskContext;
#[cfg(feature = "core_impl")]
pub use core_impl::*;
use interface::DomainType;
pub trait CoreFunction: Send + Sync {
    fn sys_alloc_pages(&self, domain_id: u64, n: usize) -> *mut u8;
    fn sys_free_pages(&self, domain_id: u64, p: *mut u8, n: usize);
    fn sys_write_console(&self, s: &str);
    fn check_kernel_space(&self, start: usize, size: usize) -> bool;
    fn sys_backtrace(&self, domain_id: u64);
    fn sys_switch_task(&self, now: *mut TaskContext, next: *const TaskContext);
    fn sys_trampoline_addr(&self) -> usize;
    fn sys_kernel_satp(&self) -> usize;
    fn sys_trap_from_user(&self) -> usize;
    fn sys_trap_to_user(&self) -> usize;
    /// This func will be deleted
    fn blk_crash_trick(&self) -> bool;
    fn sys_read_time_ms(&self) -> u64;
    fn sys_get_domain(&self, name: &str) -> Option<DomainType>;
}

#[cfg(feature = "core_impl")]
mod core_impl {
    use alloc::boxed::Box;

    use context::TaskContext;
    use interface::DomainType;
    use spin::Once;

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

    pub fn check_kernel_space(start: usize, size: usize) -> bool {
        unsafe { CORE_FUNC.get_unchecked().check_kernel_space(start, size) }
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
        unsafe { CORE_FUNC.get_unchecked().sys_trampoline_addr() }
    }

    pub fn kernel_satp() -> usize {
        unsafe { CORE_FUNC.get_unchecked().sys_kernel_satp() }
    }

    pub fn trap_from_user() -> usize {
        unsafe { CORE_FUNC.get_unchecked().sys_trap_from_user() }
    }

    pub fn trap_to_user() -> usize {
        unsafe { CORE_FUNC.get_unchecked().sys_trap_to_user() }
    }

    pub fn switch_task(now: *mut TaskContext, next: *const TaskContext) {
        unsafe { CORE_FUNC.get_unchecked().sys_switch_task(now, next) }
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
}
