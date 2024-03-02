#![no_std]

#[cfg(feature = "domain")]
#[macro_use]
pub mod console;
#[cfg(feature = "domain")]
mod frame;
#[cfg(feature = "domain")]
mod logging;

extern crate alloc;
use alloc::sync::Arc;
#[cfg(feature = "domain")]
pub use frame::{FrameTracker, FRAME_SIZE};

pub trait Syscall: Send + Sync {
    fn sys_alloc_pages(&self, domain_id: u64, n: usize) -> *mut u8;
    fn sys_free_pages(&self, domain_id: u64, p: *mut u8, n: usize);
    fn sys_write_console(&self, s: &str);
    fn backtrace(&self, domain_id: u64);
    fn read_timer(&self) -> u64;
    fn sys_get_blk_domain(&self) -> Option<Arc<dyn interface::BlkDeviceDomain>>;
    fn sys_get_uart_domain(&self) -> Option<Arc<dyn interface::UartDomain>>;
    fn sys_get_gpu_domain(&self) -> Option<Arc<dyn interface::GpuDomain>>;
    fn sys_get_input_domain(&self, ty: &str) -> Option<Arc<dyn interface::InputDomain>>;
    fn sys_get_fs_domain(&self, ty: &str) -> Option<Arc<dyn interface::FsDomain>>;
    fn sys_get_rtc_domain(&self) -> Option<Arc<dyn interface::RtcDomain>>;
    fn sys_get_cache_blk_domain(&self) -> Option<Arc<dyn interface::CacheBlkDeviceDomain>>;
}
#[cfg(feature = "domain")]
mod __impl {
    use crate::frame::FrameTracker;
    use crate::{logging, Syscall};
    use alloc::boxed::Box;
    use alloc::sync::Arc;
    use rref::domain_id;
    use spin::Once;

    static SYSCALL: Once<Box<dyn Syscall>> = Once::new();
    pub fn init(syscall: Box<dyn Syscall>) {
        SYSCALL.call_once(|| syscall);
        logging::init_logger();
        println!("syscall initialized");
    }

    pub fn alloc_raw_pages(n: usize) -> *mut u8 {
        SYSCALL
            .get()
            .expect("syscall not initialized")
            .sys_alloc_pages(domain_id(), n)
    }

    pub fn free_raw_pages(p: *mut u8, n: usize) {
        SYSCALL
            .get()
            .expect("syscall not initialized")
            .sys_free_pages(domain_id(), p, n);
    }

    pub fn alloc_pages(n: usize) -> FrameTracker {
        let raw = SYSCALL
            .get()
            .expect("syscall not initialized")
            .sys_alloc_pages(domain_id(), n);
        FrameTracker::new(raw as usize, n)
    }

    pub fn backtrace() {
        SYSCALL
            .get()
            .expect("syscall not initialized")
            .backtrace(domain_id());
    }

    pub fn write_console(s: &str) {
        SYSCALL
            .get()
            .expect("syscall not initialized")
            .sys_write_console(s);
    }

    pub fn get_blk_domain() -> Option<Arc<dyn interface::BlkDeviceDomain>> {
        SYSCALL
            .get()
            .expect("syscall not initialized")
            .sys_get_blk_domain()
    }

    pub fn get_uart_domain() -> Option<Arc<dyn interface::UartDomain>> {
        SYSCALL
            .get()
            .expect("syscall not initialized")
            .sys_get_uart_domain()
    }

    pub fn get_gpu_domain() -> Option<Arc<dyn interface::GpuDomain>> {
        SYSCALL
            .get()
            .expect("syscall not initialized")
            .sys_get_gpu_domain()
    }

    pub fn get_input_domain(ty: &str) -> Option<Arc<dyn interface::InputDomain>> {
        SYSCALL
            .get()
            .expect("syscall not initialized")
            .sys_get_input_domain(ty)
    }

    pub fn get_fs_domain(ty: &str) -> Option<Arc<dyn interface::FsDomain>> {
        SYSCALL
            .get()
            .expect("syscall not initialized")
            .sys_get_fs_domain(ty)
    }

    pub fn read_timer() -> u64 {
        SYSCALL.get().expect("syscall not initialized").read_timer()
    }

    pub fn get_rtc_domain() -> Option<Arc<dyn interface::RtcDomain>> {
        SYSCALL
            .get()
            .expect("syscall not initialized")
            .sys_get_rtc_domain()
    }

    pub fn get_cache_blk_domain() -> Option<Arc<dyn interface::CacheBlkDeviceDomain>> {
        SYSCALL
            .get()
            .expect("syscall not initialized")
            .sys_get_cache_blk_domain()
    }
}

#[cfg(feature = "domain")]
pub use __impl::*;
