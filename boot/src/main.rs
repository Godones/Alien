#![no_std]
#![no_main]

use core::arch::global_asm;
use core::hint::spin_loop;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use kernel::config::{CPU_NUM, FRAME_SIZE, TIMER_FREQ};
use kernel::driver::rtc::get_rtc_time;
use kernel::{config, driver, memory, print, println, task, thread_local_init, timer, trap};

use kernel::fs::list_dir;
use kernel::fs::vfs::init_vfs;
use riscv::register::sstatus::{set_spp, SPP};

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

/// rust_main is the entry of the kernel
#[no_mangle]
pub fn rust_main(hart_id: usize, device_tree_addr: usize) -> ! {
    unsafe {
        set_spp(SPP::Supervisor);
    }
    if hart_id == 0 {
        clear_bss();
        println!("{}", config::FLAG);
        print::init_logger();
        preprint::init_print(&print::PrePrint);
        memory::init_frame_allocator();
        memory::init_slab_system(FRAME_SIZE, 32);
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
    get_rtc_time()
        .map(|x| {
            println!("Current time:{:?}", x);
        })
        .unwrap();
    init_vfs();
    list_dir();
    task::init_process();
    task::schedule::first_into_user();
}

fn wait_all_cpu_start() {
    while CPUS.load(Ordering::Acquire) < CPU_NUM {
        spin_loop()
    }
}
