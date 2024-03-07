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
use alloc::vec::Vec;
use core::ops::Range;
use downcast_rs::{impl_downcast, DowncastSync};
#[cfg(feature = "domain")]
pub use frame::{FrameTracker, FRAME_SIZE};

pub trait KTask: Send + Sync + DowncastSync {
    fn to_wait(&self);
    fn to_wakeup(&self);
    fn have_signal(&self) -> bool;
}

impl_downcast!(sync KTask);

pub trait KTaskShim: Send + Sync {
    fn get_task(&self) -> Arc<dyn KTask>;
    fn put_task(&self, task: Arc<dyn KTask>);
    fn suspend(&self);
    fn transfer_ptr_raw(&self, ptr: usize) -> usize;
    fn transfer_buf_raw(&self, src: usize, size: usize) -> Vec<&mut [u8]>;
}

#[allow(unused)]
impl dyn KTaskShim {
    fn copy_data_to_task<T: 'static + Copy>(&self, src: *const T, dst: *mut T) {
        let size = core::mem::size_of::<T>();
        let bufs = self.transfer_buf_raw(dst as usize, size);
        let src = unsafe { core::slice::from_raw_parts(src as *const u8, size) };
        let mut start = 0;
        for buffer in bufs {
            let len = if start + buffer.len() > size {
                size - start
            } else {
                buffer.len()
            };
            unsafe {
                core::ptr::copy_nonoverlapping(src.as_ptr().add(start), buffer.as_mut_ptr(), len);
            }
            start += len;
        }
    }
    fn copy_data_from_task<T: 'static + Copy>(&self, src: *const T, dst: *mut T) {
        let size = core::mem::size_of::<T>();
        let bufs = self.transfer_buf_raw(src as usize, size);
        let dst = unsafe { core::slice::from_raw_parts_mut(dst as *mut u8, size) };
        let mut start = 0;
        for buffer in bufs {
            let len = if start + buffer.len() > size {
                size - start
            } else {
                buffer.len()
            };
            unsafe {
                core::ptr::copy_nonoverlapping(buffer.as_ptr(), dst.as_mut_ptr().add(start), len);
            }
            start += len;
        }
    }
    fn transfer_ptr_mut<T>(&self, ptr: *mut T) -> &'static mut T {
        let ptr = ptr as usize;
        let ptr = self.transfer_ptr_raw(ptr);
        unsafe { &mut *(ptr as *mut T) }
    }
    fn transfer_ptr<T>(&self, ptr: *const T) -> &'static T {
        let ptr = ptr as usize;
        let ptr = self.transfer_ptr_raw(ptr);
        unsafe { &*(ptr as *const T) }
    }
}

pub enum DeviceType {
    Block,
    Uart,
    Gpu,
    Input,
    Rtc,
}

#[allow(unused)]
pub enum DomainType<'a> {
    Block,
    Uart,
    Gpu,
    Input(&'a str),
    Rtc,
    CacheBlock,
}

pub trait Syscall: Send + Sync {
    fn sys_alloc_pages(&self, domain_id: u64, n: usize) -> *mut u8;
    fn sys_free_pages(&self, domain_id: u64, p: *mut u8, n: usize);
    fn sys_write_console(&self, s: &str);
    fn sys_backtrace(&self, domain_id: u64);
    fn sys_read_timer(&self) -> u64;
    fn sys_device_space(&self, ty: DeviceType) -> Option<Range<usize>>;
    fn check_kernel_space(&self, start: usize, size: usize) -> bool;
    fn sys_get_blk_domain(&self) -> Option<Arc<dyn interface::BlkDeviceDomain>>;
    fn sys_get_shadow_blk_domain(&self) -> Option<Arc<dyn interface::BlkDeviceDomain>>;
    fn sys_get_uart_domain(&self) -> Option<Arc<dyn interface::UartDomain>>;
    fn sys_get_gpu_domain(&self) -> Option<Arc<dyn interface::GpuDomain>>;
    fn sys_get_input_domain(&self, ty: &str) -> Option<Arc<dyn interface::InputDomain>>;
    fn sys_get_rtc_domain(&self) -> Option<Arc<dyn interface::RtcDomain>>;
    fn sys_get_cache_blk_domain(&self) -> Option<Arc<dyn interface::CacheBlkDeviceDomain>>;

    /// This func will be deleted
    fn blk_crash_trick(&self) -> bool;
}
#[cfg(feature = "domain")]
mod __impl {
    use crate::frame::FrameTracker;
    use crate::{logging, DeviceType, KTask, KTaskShim, Syscall};
    use alloc::boxed::Box;
    use alloc::sync::Arc;
    use core::ops::Range;
    use rref::domain_id;
    use spin::Once;

    static KTASK_SHIM: Once<Box<dyn KTaskShim>> = Once::new();

    static SYSCALL: Once<Box<dyn Syscall>> = Once::new();

    /// Initialize the syscall interface.
    pub fn init(syscall: Box<dyn Syscall>, ktask_shim: Box<dyn KTaskShim>) {
        SYSCALL.call_once(|| syscall);
        KTASK_SHIM.call_once(|| ktask_shim);
        logging::init_logger();
        println!("syscall initialized");
    }

    /// Get the current task.
    pub fn current_task() -> Arc<dyn KTask> {
        KTASK_SHIM
            .get()
            .expect("ktask_shim not initialized")
            .get_task()
    }

    /// Put the task back to the task queue.
    pub fn put_task(task: Arc<dyn KTask>) {
        KTASK_SHIM
            .get()
            .expect("ktask_shim not initialized")
            .put_task(task);
    }

    /// Suspend the current task.
    pub fn suspend() {
        KTASK_SHIM
            .get()
            .expect("ktask_shim not initialized")
            .suspend();
    }

    pub fn copy_data_to_task<T: 'static + Copy>(src: *const T, dst: *mut T) {
        KTASK_SHIM
            .get()
            .expect("ktask_shim not initialized")
            .copy_data_to_task(src, dst);
    }

    pub fn copy_data_from_task<T: 'static + Copy>(src: *const T, dst: *mut T) {
        KTASK_SHIM
            .get()
            .expect("ktask_shim not initialized")
            .copy_data_from_task(src, dst);
    }

    pub fn transfer_ptr_mut<T>(ptr: *mut T) -> &'static mut T {
        KTASK_SHIM
            .get()
            .expect("ktask_shim not initialized")
            .transfer_ptr_mut(ptr)
    }

    pub fn transfer_ptr<T>(ptr: *const T) -> &'static T {
        KTASK_SHIM
            .get()
            .expect("ktask_shim not initialized")
            .transfer_ptr(ptr)
    }

    pub fn check_kernel_space(start: usize, size: usize) -> bool {
        SYSCALL
            .get()
            .expect("syscall not initialized")
            .check_kernel_space(start, size)
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
            .sys_backtrace(domain_id());
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

    pub fn get_shadow_blk_domain() -> Option<Arc<dyn interface::BlkDeviceDomain>> {
        SYSCALL
            .get()
            .expect("syscall not initialized")
            .sys_get_shadow_blk_domain()
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
    pub fn read_timer() -> u64 {
        SYSCALL
            .get()
            .expect("syscall not initialized")
            .sys_read_timer()
    }

    pub fn get_device_space(ty: DeviceType) -> Option<Range<usize>> {
        SYSCALL
            .get()
            .expect("syscall not initialized")
            .sys_device_space(ty)
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

    pub fn blk_crash_trick() -> bool {
        SYSCALL
            .get()
            .expect("syscall not initialized")
            .blk_crash_trick()
    }
}

#[cfg(feature = "domain")]
pub use __impl::*;
