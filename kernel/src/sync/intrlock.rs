//! interrupt-safe locks

use crate::arch;
use core::sync::atomic::AtomicIsize;

#[derive(Debug)]
pub struct IntrLock {
    is_enabled_before: bool,
    cnt: AtomicIsize,
    hart_id: usize,
}
impl IntrLock {
    pub const fn new() -> Self {
        Self {
            is_enabled_before: false,
            cnt: AtomicIsize::new(0),
            hart_id: 999,
        }
    }
    pub fn lock(&mut self) -> IntrLockGuard {
        let before_enabled = arch::is_interrupt_enable();
        // disable interrupt
        arch::interrupt_disable();
        if self.cnt.load(core::sync::atomic::Ordering::Relaxed) == 0 {
            self.hart_id = arch::hart_id();
            self.is_enabled_before = before_enabled;
        } else {
            if before_enabled {
                panic!("lock held but intr enabled");
            }
            if self.hart_id != arch::hart_id() {
                panic!("lock held by another hart");
            }
        }
        self.cnt.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        IntrLockGuard { lock: self }
    }
}

pub struct IntrLockGuard<'a> {
    lock: &'a IntrLock,
}

impl Drop for IntrLockGuard<'_> {
    fn drop(&mut self) {
        if arch::is_interrupt_enable() {
            panic!("intr enabled when drop IntrLockGuard");
        }
        if arch::hart_id() != self.lock.hart_id {
            panic!("unlock on different hart");
        }
        self.lock
            .cnt
            .fetch_sub(1, core::sync::atomic::Ordering::Relaxed);
        let val = self.lock.cnt.load(core::sync::atomic::Ordering::Relaxed);
        if val < 0 {
            panic!("lock cnt < 0");
        }
        if val == 0 && self.lock.is_enabled_before {
            arch::interrupt_enable();
        }
    }
}
unsafe impl Sync for IntrLock {}

unsafe impl Send for IntrLock {}
