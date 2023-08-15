#![no_std]
#![allow(unused)]
extern crate alloc;

use core::any::Any;
use core::ops::{Deref, DerefMut};
use spin::{Mutex, MutexGuard};

pub mod basic;
pub mod complex;

/// TODO The user should implement the `UPIntrFreeCell` trait
pub struct UPIntrFreeCell<T> {
    inner: Mutex<T>,
}

pub struct UPIntrRefMut<'a, T>(Option<MutexGuard<'a, T>>);

impl<T> UPIntrFreeCell<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Mutex::new(value),
        }
    }
    pub fn exclusive_access(&self) -> UPIntrRefMut<T> {
        let inner = self.inner.lock();
        UPIntrRefMut(Some(inner))
    }
}

impl<'a, T> Deref for UPIntrRefMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap().deref()
    }
}
impl<'a, T> DerefMut for UPIntrRefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap().deref_mut()
    }
}

pub trait GPUDevice: Send + Sync + Any {
    fn update_cursor(&self);
    fn get_frame_buffer(&self) -> &mut [u8];
    fn flush(&self);
    fn get_resolution(&self) -> (u32, u32);
}
