use core::cell::UnsafeCell;
use core::hint::spin_loop;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::AtomicBool;

pub struct Mutex<T>{
    lock:AtomicBool,
    data:UnsafeCell<T>
}

impl <T> Mutex<T>{
    pub const fn new(data:T)->Self{
        Self{
            lock:AtomicBool::new(false),
            data:UnsafeCell::new(data),
        }
    }
    pub fn lock(&self)->MutexGuard<T>{
        while self.lock.compare_exchange(false,true,core::sync::atomic::Ordering::Acquire,core::sync::atomic::Ordering::Relaxed).is_err(){
            spin_loop();
        }
        MutexGuard{
            lock:self,
            data:unsafe{&mut *self.data.get()},
        }
    }
}

pub struct MutexGuard<'a,T>{
    lock:&'a Mutex<T>,
    data:&'a mut T,
}

impl <T> Drop for MutexGuard<'_,T>{
    fn drop(&mut self) {
        self.lock.lock.store(false,core::sync::atomic::Ordering::Release);
    }
}

impl <T> Deref for MutexGuard<'_,T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl <T> DerefMut for MutexGuard<'_,T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data
    }
}
