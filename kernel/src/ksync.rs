use kernel_sync::TicketMutexGuard;

pub type SpinMutex<T> = kernel_sync::SpinMutex<T>;
pub type TicketMutex<T> = kernel_sync::TicketMutex<T>;
pub type RwLock<T> = kernel_sync::RwLock<T>;
pub type Mutex<T> = TicketMutex<T>;
pub type MutexGuard<'a, T> = TicketMutexGuard<'a, T>;
