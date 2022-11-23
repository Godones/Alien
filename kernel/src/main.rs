#![no_std]
#![no_main]
#![feature(core_intrinsics)]
#![feature(panic_info_message)]
#![feature(default_alloc_error_handler)]
#![feature(naked_functions)]
#![feature(asm_const)]
#![feature(const_for)]
#![feature(const_cmp)]
#![feature(const_mut_refs)]
use crate::config::FRAME_SIZE;
use core::arch::global_asm;
use cfg_if::cfg_if;

#[macro_use]
mod console;
mod arch;
mod config;
mod driver;
mod logging;
mod mm;
mod panic;
mod sbi;
mod trap;

// extern crate alloc;
#[macro_use]
extern crate log;
extern crate alloc;

global_asm!(include_str!("boot/boot.asm"));

/// rust 入口函数
///
/// 进行操作系统的初始化，
#[no_mangle]
pub extern "C" fn rust_main(_hart_id: usize, _device_tree_addr: usize) -> ! {
    println!("{}",config::FLAG);
    logging::init_logger();
    preprint::init_print(&console::PrePrint);
    mm::init_frame_allocator();
    mm::init_slab_system(FRAME_SIZE, 32);
    cfg_if!(
        if #[cfg(feature = "test")] {
            mm::test_simple_bitmap();
            mm::frame_allocator_test();
            mm::test_heap();
            mm::test_page_allocator();
        }
    );
    mm::build_kernel_address_space();
    mm::activate_paging_mode();
    panic!("正常关机")
}
