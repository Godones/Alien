use super::{Mutex, MutexGuard};

pub struct SleepLock<T> {
    lock: Mutex<T>,
    condvar: Condvar,
}

struct Condvar;

struct SleepLockGuard<'a, T> {
    lock: &'a SleepLock<T>,
}

