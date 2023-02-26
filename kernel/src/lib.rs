#![no_std]
#![feature(core_intrinsics)]
#![feature(naked_functions)]
#![feature(let_chains)]
#![feature(error_in_core)]
#![feature(associated_type_bounds)]
#![allow(semicolon_in_expressions_from_macros)]
#[macro_use]
pub mod print;
pub mod arch;
pub mod config;
pub mod driver;
pub mod fs;
pub mod memory;
pub mod sbi;
pub mod task;
pub mod timer;
pub mod trap;
mod syscall;


// extern crate alloc;
#[macro_use]
extern crate log;
extern crate alloc;

pub fn thread_local_init() {
    // 允许内核读写用户态内存
    // 取决于 CPU 的 RISC-V 规范版本就行处理
    unsafe {
        riscv::register::sstatus::set_sum();
    }
}
