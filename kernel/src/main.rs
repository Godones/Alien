// remove std lib
#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(default_alloc_error_handler)]
#![feature(naked_functions)]
#![feature(asm_const)]
#![feature(asm_sym)]
#![allow(unaligned_references)]

use core::arch::asm;

#[macro_use]
mod console;
mod panic;
mod sbi;
mod logging;
mod driver;
mod arch;
mod config;
mod mm;


// extern crate alloc;
#[macro_use]
extern crate log;
/// 汇编入口函数
///
/// 分配栈 并调到rust入口函数
#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
unsafe extern "C" fn _start() -> ! {
    const STACK_SIZE: usize = 4096;

    #[link_section = ".bss.stack"]
    static mut STACK: [u8; STACK_SIZE] = [0u8; STACK_SIZE];

    asm!(
    "   la  sp, {stack} + {stack_size}
            j   rust_main
        ",
    stack_size = const STACK_SIZE,
    stack      =   sym STACK,
    options(noreturn),
    )
}

/// rust 入口函数
///
/// 进行操作系统的初始化，
#[no_mangle]
pub extern "C" fn rust_main(hart_id: usize, _device_tree_addr: usize) -> ! {
    // 让其他核心进入等待
    if hart_id != 0 {
        support_hart_resume(hart_id, 0);
    }
    logging::init();
    // 调用rust api关机
    panic!("正常关机")
}


/// 辅助核心进入的函数
///
/// 目前让除 0 核之外的其他内核进入该函数进行等待
#[allow(unused)]
extern "C" fn support_hart_resume(hart_id: usize, _param: usize) {
    loop {
        // 使用wfi 省电
        unsafe { asm!("wfi") }
    }
}
