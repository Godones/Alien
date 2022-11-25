#![no_std]
#![no_main]
#![feature(core_intrinsics)]
#![feature(panic_info_message)]
#![feature(default_alloc_error_handler)]

#[macro_use]
mod print;
mod config;
mod driver;
mod memory;
mod panic;
mod sbi;
mod timer;
mod trap;

// extern crate alloc;
#[macro_use]
extern crate log;
extern crate alloc;

use crate::config::{FRAME_SIZE, TIMER_FREQ};
use crate::timer::read_timer;
use cfg_if::cfg_if;
use core::arch::global_asm;

global_asm!(include_str!("boot/boot.asm"));

/// rust 入口函数
///
/// 进行操作系统的初始化，
#[no_mangle]
pub extern "C" fn rust_main(_hart_id: usize, _device_tree_addr: usize) -> ! {
    println!("{}", config::FLAG);
    print::init_logger();
    preprint::init_print(&print::PrePrint);
    memory::init_frame_allocator();
    memory::init_slab_system(FRAME_SIZE, 32);
    cfg_if!(
        if #[cfg(feature = "test")] {
            memory::test_simple_bitmap();
            memory::frame_allocator_test();
            memory::test_heap();
            memory::test_page_allocator();
        }
    );
    memory::build_kernel_address_space();
    memory::activate_paging_mode();
    let time = read_timer();
    println!("time: {}", time);
    trap::init_trap_subsystem();
    timer::set_next_trigger(TIMER_FREQ);
    loop {}
}
