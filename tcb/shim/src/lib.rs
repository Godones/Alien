#![no_std]
#![allow(unused)]
extern crate alloc;

use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use downcast_rs::{impl_downcast, DowncastSync};
use spin::Once;

pub trait KTask: Send + Sync + DowncastSync {
    fn to_wait(&self);
    fn to_wakeup(&self);
    fn have_signal(&self) -> bool;
}

impl_downcast!(sync KTask);

pub trait KTaskShim: Send + Sync {
    fn take_current_task(&self) -> Option<Arc<dyn KTask>>;
    fn current_task(&self) -> Option<Arc<dyn KTask>>;
    fn put_task(&self, task: Arc<dyn KTask>);
    fn suspend(&self);
    fn schedule_now(&self, task: Arc<dyn KTask>);
    fn transfer_ptr_raw(&self, ptr: usize) -> usize;
    fn transfer_buf_raw(&self, src: usize, size: usize) -> Vec<&mut [u8]>;
}

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

static KTASK_SHIM: Once<Box<dyn KTaskShim>> = Once::new();

#[cfg(feature = "kernel")]
pub fn register_task_func(task_shim: Box<dyn KTaskShim>) {
    KTASK_SHIM.call_once(|| task_shim);
}

#[cfg(feature = "lib")]
/// Get the current task.
pub fn take_current_task() -> Option<Arc<dyn KTask>> {
    KTASK_SHIM
        .get()
        .expect("ktask_shim not initialized")
        .take_current_task()
}
#[cfg(feature = "lib")]
pub fn current_task() -> Option<Arc<dyn KTask>> {
    KTASK_SHIM
        .get()
        .expect("ktask_shim not initialized")
        .current_task()
}
#[cfg(feature = "lib")]
pub fn suspend() {
    KTASK_SHIM
        .get()
        .expect("ktask_shim not initialized")
        .suspend()
}
#[cfg(feature = "lib")]
/// Put the task back to the task queue.
pub fn put_task(task: Arc<dyn KTask>) {
    KTASK_SHIM
        .get()
        .expect("ktask_shim not initialized")
        .put_task(task);
}
#[cfg(feature = "lib")]
/// Suspend the current task.
pub fn schedule_now(task: Arc<dyn KTask>) {
    KTASK_SHIM
        .get()
        .expect("ktask_shim not initialized")
        .schedule_now(task);
}
#[cfg(feature = "lib")]
pub fn copy_data_to_task<T: 'static + Copy>(src: *const T, dst: *mut T) {
    KTASK_SHIM
        .get()
        .expect("ktask_shim not initialized")
        .copy_data_to_task(src, dst);
}
#[cfg(feature = "lib")]
pub fn copy_data_from_task<T: 'static + Copy>(src: *const T, dst: *mut T) {
    KTASK_SHIM
        .get()
        .expect("ktask_shim not initialized")
        .copy_data_from_task(src, dst);
}
#[cfg(feature = "lib")]
pub fn transfer_ptr_mut<T>(ptr: *mut T) -> &'static mut T {
    KTASK_SHIM
        .get()
        .expect("ktask_shim not initialized")
        .transfer_ptr_mut(ptr)
}
#[cfg(feature = "lib")]
pub fn transfer_ptr<T>(ptr: *const T) -> &'static T {
    KTASK_SHIM
        .get()
        .expect("ktask_shim not initialized")
        .transfer_ptr(ptr)
}
