//! RCU (Read-Copy-Update) is a synchronization mechanism that allows for efficient read access to shared data structures.

use alloc::boxed::Box;
use core::{arch::asm, cell::UnsafeCell};

pub fn synchronize_sched() {
    synchronize_rcu();
}

// /// Acquire the RCU read lock
// pub fn rcu_read_lock_sched() {
//     // preempt_disable();
// }
//
// /// Release the RCU read lock
// pub fn rcu_read_unlock_sched() {
//     // preempt_enable();
// }

pub fn synchronize_rcu() {
    crate::task::synchronize_rcu();
}

#[derive(Debug)]
pub struct RcuData<T> {
    data_ptr: UnsafeCell<*mut T>,
}

unsafe impl<T> Sync for RcuData<T> {}
unsafe impl<T> Send for RcuData<T> {}

impl<T> RcuData<T> {
    pub fn new(val: Box<T>) -> Self {
        let data_ptr = Box::into_raw(val);
        // make sure the size of the pointer is 8
        assert_eq!(core::mem::size_of_val(&data_ptr), 8);
        Self {
            data_ptr: UnsafeCell::new(data_ptr),
        }
    }
    pub fn get(&self) -> &T {
        let ptr_ptr = self.data_ptr.get();
        unsafe { &*(ptr_ptr.read_volatile()) }
    }

    pub fn swap(&self, val: Box<T>) -> Box<T> {
        let data_ptr = Box::into_raw(val);
        let old_data_ptr = unsafe { *self.data_ptr.get() };
        // This should insert a memory barrier
        // smp_wmb()
        unsafe {
            #[cfg(target_arch = "riscv64")]
            asm!("fence rw,w",);
            // update the ptr
            self.data_ptr.get().write_volatile(data_ptr);
        }
        // self.data_ptr = data_ptr;
        // after this, the readers can read the new data
        unsafe { Box::from_raw(old_data_ptr) }
    }
}

#[macro_export]
macro_rules! read_once {
    ($val:expr) => {
        unsafe { core::ptr::read_volatile($val) }
    };
}

#[macro_export]
macro_rules! write_once {
    ($val:expr, $data:expr) => {
        unsafe { core::ptr::write_volatile($val, $data) }
    };
}
