//! 内核的主要代码，包含各个子模块的实现。
//!
//! 通过导出子模块，boot模块可以调用内核的各个子模块完成初始化工作

#![no_std]
#![feature(core_intrinsics)]
#![feature(error_in_core)]
#![feature(panic_info_message)]
#![feature(atomic_from_mut)]
#![feature(ip_in_core)]
#![feature(stmt_expr_attributes)]
#![feature(addr_parse_ascii)]
#![feature(let_chains)]
#![feature(trait_upcasting)]
// #![deny(missing_docs)]

extern crate alloc;
#[macro_use]
extern crate log;

// extern crate syscall_table;

use spin::Once;

use basemachine::MachineInfo;

#[macro_use]
pub mod print;
pub mod arch;
pub mod board;
pub mod config;
pub mod device;
pub mod driver;
pub mod error;
pub mod fs;
pub mod gui;
pub mod interrupt;
pub mod ipc;
pub mod memory;
pub mod net;
pub mod panic;
pub mod sbi;
pub mod sys;
pub mod system;
pub mod task;
pub mod timer;
pub mod trace;
pub mod trap;

#[macro_use]
extern crate syscall_table;

pub use syscall_table::*;

/// 设备基本信息
pub static MACHINE_INFO: Once<MachineInfo> = Once::new();

/// 初始化设备基本信息
///
/// 在后续的代码中，可以通过调用`MACHINE_INFO.get()`来获取设备基本信息
pub fn init_machine_info(machine_info: MachineInfo) {
    MACHINE_INFO.call_once(|| machine_info);
}

/// 允许内核读写用户态内存。
/// 取决于 CPU 的 RISC-V 规范版本就行处理
pub fn thread_local_init() {
    unsafe {
        riscv::register::sstatus::set_sum();
    }
}
