#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(default_alloc_error_handler)]
mod panic;

use cfg_if::cfg_if;
use core::arch::global_asm;
use core::hint::spin_loop;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use kernel::config::{CPU_NUM, FRAME_SIZE, TIMER_FREQ};
use kernel::sbi::shutdown;
use kernel::{config, driver, fs, memory, print, println, thread_local_init, timer, trap};
use kernel::driver::rtc::get_rtc_time;

global_asm!(include_str!("./boot.asm"));
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
        println!("{}", config::FLAG);
        print::init_logger();
        // preprint::init_print(&print::PrePrint);
        memory::init_frame_allocator();
        memory::init_slab_system(FRAME_SIZE, 32);
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
    get_rtc_time().map(|x|{println!("Current time:{:?}", x);}).is_none().then(||{println!("time error");});
    // 文件系统测试
    fs::fs_repl();
    // fs::dbfs_test();
    loop {
        shutdown();
    }
}

fn wait_all_cpu_start() {
    while CPUS.load(Ordering::Acquire) < CPU_NUM {
        spin_loop()
    }
}
