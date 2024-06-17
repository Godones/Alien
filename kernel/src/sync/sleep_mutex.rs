use ksync::{Mutex, MutexGuard};

use crate::task::yield_now;

#[derive(Debug)]
pub struct SleepMutex<T> {
    mutex: Mutex<T>,
}

impl<T> SleepMutex<T> {
    pub fn new(data: T) -> Self {
        Self {
            mutex: Mutex::new(data),
        }
    }

    pub fn lock(&self) -> MutexGuard<T> {
        loop {
            match self.mutex.try_lock() {
                Some(guard) => return guard,
                None => yield_now(),
            }
        }
    }
}
