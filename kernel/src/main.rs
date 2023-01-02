#![no_std]
#![no_main]
#![feature(core_intrinsics)]
#![feature(panic_info_message)]
#![feature(default_alloc_error_handler)]
#![feature(naked_functions)]
#![feature(let_chains)]
#![feature(error_in_core)]
#![feature(associated_type_bounds)]
#![allow(semicolon_in_expressions_from_macros)]
#[macro_use]
mod print;
mod arch;
mod config;
mod driver;
mod fs;
mod memory;
mod panic;
mod sbi;
mod task;
mod timer;
mod trap;

// extern crate alloc;
#[macro_use]
extern crate log;
extern crate alloc;

use crate::config::{CPU_NUM, FRAME_SIZE, TIMER_FREQ};
use crate::sbi::shutdown;
use cfg_if::cfg_if;
use core::arch::global_asm;
use core::hint::spin_loop;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

global_asm!(include_str!("boot/boot.asm"));

// 多核启动标志
static STARTED: AtomicBool = AtomicBool::new(false);

static CPUS: AtomicUsize = AtomicUsize::new(0);

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

/// rust 入口函数
///
/// 进行操作系统的初始化，
#[no_mangle]
pub fn rust_main(hart_id: usize, device_tree_addr: usize) -> ! {
    if hart_id == 0 {
        clear_bss();
        print::init_logger();
        preprint::init_print(&print::PrePrint);
        memory::init_frame_allocator();
        memory::init_slab_system(FRAME_SIZE, 32);
        println!("{}", config::FLAG);
        cfg_if!(
        if #[cfg(feature = "test")] {
            memory::test_simple_bitmap();
            memory::frame_allocator_test();
            memory::test_heap();
            memory::test_page_allocator();
            fs::test_gmanager();
        }
        );
        memory::build_kernel_address_space();
        memory::activate_paging_mode();
        thread_local_init();
        trap::init_trap_subsystem();
        // timer::set_next_trigger(TIMER_FREQ);
        CPUS.fetch_add(1, Ordering::Release);
        STARTED.store(true, Ordering::Relaxed);
    } else {
        while !STARTED.load(Ordering::Relaxed) {
            spin_loop();
        }
        memory::activate_paging_mode();
        thread_local_init();
        trap::init_trap_subsystem();
        timer::set_next_trigger(TIMER_FREQ);
        CPUS.fetch_add(1, Ordering::Release);
    }
    // 等待其它cpu核启动
    wait_all_cpu_start();
    // 设备树初始化
    driver::init_dt(device_tree_addr);
    // 文件系统测试
    fs::fs_repl();
    // fs::dbfs_test();
    loop {
        shutdown();
    }
}

pub fn thread_local_init() {
    // 允许内核读写用户态内存
    // 取决于 CPU 的 RISC-V 规范版本就行处理
    unsafe {
        riscv::register::sstatus::set_sum();
    }
}

fn wait_all_cpu_start() {
    while CPUS.load(Ordering::Acquire) < CPU_NUM {
        spin_loop()
    }
}
