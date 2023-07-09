#![no_std]
#![feature(core_intrinsics)]
#![feature(error_in_core)]
#![feature(panic_info_message)]

extern crate alloc;
#[macro_use]
extern crate log;

#[macro_use]
pub mod print;
pub mod arch;
pub mod board;
pub mod config;
pub mod driver;
mod error;
pub mod fs;
mod gui;
pub mod net;
pub mod ipc;
pub mod memory;
mod panic;
pub mod sbi;
mod sync;
pub mod syscall;
pub mod system;
pub mod task;
pub mod timer;
mod trace;
pub mod trap;

pub fn thread_local_init() {
    // 允许内核读写用户态内存
    // 取决于 CPU 的 RISC-V 规范版本就行处理
    unsafe {
        riscv::register::sstatus::set_sum();
    }
}
