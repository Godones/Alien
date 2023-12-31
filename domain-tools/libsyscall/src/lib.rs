#![no_std]

#[macro_use]
pub mod console;
mod logging;
extern crate alloc;

use alloc::boxed::Box;
use spin::Once;

static SYSCALL: Once<Box<dyn Syscall>> = Once::new();
static CRATE_DOMAIN_ID: Once<u64> = Once::new();

pub trait Syscall: Send + Sync {
    fn sys_alloc_pages(&self, domain_id: u64, n: usize) -> *mut u8;
    fn sys_free_pages(&self, domain_id: u64, p: *mut u8, n: usize);
    fn sys_write_console(&self, s: &str);
    fn backtrace(&self, domain_id: u64);
}

pub fn init(syscall: Box<dyn Syscall>, domain_id: u64) {
    SYSCALL.call_once(|| syscall);
    CRATE_DOMAIN_ID.call_once(|| domain_id);
    logging::init_logger();
    println!("syscall initialized");
}

pub fn alloc_pages(n: usize) -> *mut u8 {
    SYSCALL
        .get()
        .expect("syscall not initialized")
        .sys_alloc_pages(domain_id(), n)
}

pub fn free_pages(p: *mut u8, n: usize) {
    SYSCALL
        .get()
        .expect("syscall not initialized")
        .sys_free_pages(domain_id(), p, n);
}

#[inline]
pub fn domain_id() -> u64 {
    *CRATE_DOMAIN_ID.get().expect("domain id not initialized")
}

pub fn backtrace() {
    SYSCALL
        .get()
        .expect("syscall not initialized")
        .backtrace(domain_id());
}

pub fn write_console(s: &str) {
    SYSCALL
        .get()
        .expect("syscall not initialized")
        .sys_write_console(s);
}
