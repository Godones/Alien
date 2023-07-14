#![no_std]
#![feature(core_intrinsics)]
#![feature(error_in_core)]
#![feature(panic_info_message)]
#![feature(atomic_from_mut)]

extern crate alloc;
#[macro_use]
extern crate log;

use spin::Once;

use basemachine::MachineInfo;

#[macro_use]
pub mod print;
pub mod arch;
pub mod board;
pub mod config;
pub mod driver;
mod error;
pub mod fs;
mod gui;
pub mod ipc;
pub mod memory;
pub mod net;
mod panic;
pub mod sbi;
mod sync;
pub mod sys;
pub mod syscall;
pub mod system;
pub mod task;
pub mod timer;
mod trace;
pub mod trap;

pub static MACHINE_INFO: Once<MachineInfo> = Once::new();

pub fn init_machine_info(machine_info: MachineInfo) {
    MACHINE_INFO.call_once(|| machine_info);
}

pub fn thread_local_init() {
    // 允许内核读写用户态内存
    // 取决于 CPU 的 RISC-V 规范版本就行处理
    unsafe {
        riscv::register::sstatus::set_sum();
    }
}
