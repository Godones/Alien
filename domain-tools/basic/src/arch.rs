use core::cell::UnsafeCell;
use core::ops::Deref;

pub fn hart_id() -> usize {
    arch::hart_id()
}

pub struct CpuLocal<T>(UnsafeCell<T>);

unsafe impl<T> Sync for CpuLocal<T> {}

impl<T> CpuLocal<T> {
    pub const fn new(value: T) -> Self {
        Self(UnsafeCell::new(value))
    }
}

impl<T> Deref for CpuLocal<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.0.get() }
    }
}
