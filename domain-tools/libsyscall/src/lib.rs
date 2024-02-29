#![no_std]

#[macro_use]
pub mod console;
mod logging;
extern crate alloc;

use alloc::boxed::Box;
use alloc::sync::Arc;
use rref::domain_id;
use spin::Once;

static SYSCALL: Once<Box<dyn Syscall>> = Once::new();
pub trait Syscall: Send + Sync {
    fn sys_alloc_pages(&self, domain_id: u64, n: usize) -> *mut u8;
    fn sys_free_pages(&self, domain_id: u64, p: *mut u8, n: usize);
    fn sys_write_console(&self, s: &str);
    fn backtrace(&self, domain_id: u64);
    fn sys_get_blk_domain(&self) -> Option<Arc<dyn interface::BlkDevice>>;
    fn sys_get_uart_domain(&self) -> Option<Arc<dyn interface::Uart>>;
}

pub fn init(syscall: Box<dyn Syscall>) {
    SYSCALL.call_once(|| syscall);
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

pub fn get_blk_domain() -> Option<Arc<dyn interface::BlkDevice>> {
    SYSCALL
        .get()
        .expect("syscall not initialized")
        .sys_get_blk_domain()
}

pub fn get_uart_domain() -> Option<Arc<dyn interface::Uart>> {
    SYSCALL
        .get()
        .expect("syscall not initialized")
        .sys_get_uart_domain()
}
