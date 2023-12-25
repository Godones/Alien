#![no_std]

mod sheap;
mod syscall;

extern crate alloc;
use core::sync::atomic::AtomicU64;

pub use sheap::SharedHeapAllocator;
pub use syscall::DomainSyscall;
static DOMAIN_IDS: AtomicU64 = AtomicU64::new(0);

pub fn alloc_domain_id() -> u64 {
    DOMAIN_IDS.fetch_add(1, core::sync::atomic::Ordering::SeqCst)
}
