//! interrupt-safe locks

use crate::arch;
use core::cell::{RefCell, RefMut};
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, AtomicIsize, AtomicUsize};

#[derive(Debug)]
pub struct IntrLock<T> {
    is_enabled_before: AtomicBool,
    cnt: AtomicIsize,
    hart_id: AtomicUsize,
    data: RefCell<T>,
}
impl<T> IntrLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            is_enabled_before: AtomicBool::new(false),
            cnt: AtomicIsize::new(0),
            hart_id: AtomicUsize::new(0),
            data: RefCell::new(data),
        }
    }
    pub fn lock(&self) -> IntrLockGuard<T> {
        let before_enabled = arch::is_interrupt_enable();
        // disable interrupt
        if before_enabled {
            arch::interrupt_disable();
        }
        if self.cnt.load(core::sync::atomic::Ordering::Relaxed) == 0 {
            self.hart_id
                .swap(arch::hart_id(), core::sync::atomic::Ordering::Relaxed);
            self.is_enabled_before
                .store(before_enabled, core::sync::atomic::Ordering::Relaxed);
        } else {
            if before_enabled {
                panic!("lock held but intr enabled");
            }
            if self.hart_id.load(core::sync::atomic::Ordering::Relaxed) != arch::hart_id() {
                panic!("lock held by another hart");
            }
        }
        let value = self.cnt.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        if value > 0 {
            panic!("lock held by another hart");
        }
        IntrLockGuard {
            lock: self,
            data: Some(self.data.borrow_mut()),
        }
    }
}

pub struct IntrLockGuard<'a, T> {
    lock: &'a IntrLock<T>,
    data: Option<RefMut<'a, T>>,
}

impl<T> Deref for IntrLockGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.data.as_ref().unwrap().deref()
    }
}

impl<T> DerefMut for IntrLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data.as_mut().unwrap().deref_mut()
    }
}

impl<T> Drop for IntrLockGuard<'_, T> {
    fn drop(&mut self) {
        if arch::is_interrupt_enable() {
            panic!("intr enabled when drop IntrLockGuard");
        }
        if arch::hart_id()
            != self
                .lock
                .hart_id
                .load(core::sync::atomic::Ordering::Relaxed)
        {
            panic!("unlock on different hart");
        }
        self.lock
            .cnt
            .fetch_sub(1, core::sync::atomic::Ordering::Relaxed);
        let val = self.lock.cnt.load(core::sync::atomic::Ordering::Relaxed);
        if val < 0 {
            panic!("lock cnt < 0");
        }
        self.data = None;
        if val == 0
            && self
                .lock
                .is_enabled_before
                .load(core::sync::atomic::Ordering::Relaxed)
        {
            arch::interrupt_enable();
        }
    }
}
unsafe impl<T> Sync for IntrLock<T> {}

unsafe impl<T> Send for IntrLock<T> {}
