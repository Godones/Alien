#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(asm_const)]

use core::arch::asm;
use core::hint::spin_loop;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use riscv::register::sstatus::{set_spp, SPP};

use basemachine::machine_info_from_dtb;
use kernel::config::{CPU_NUM, STACK_SIZE};
use kernel::fs::vfs::init_vfs;
use kernel::memory::{init_memory_system, kernel_info};
use kernel::print::init_print;
use kernel::sbi::hart_start;
use kernel::task::init_per_cpu;
use kernel::{config, driver, println, syscall, task, thread_local_init, timer, trap};

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

#[link_section = ".bss.stack"]
static mut STACK: [u8; STACK_SIZE * CPU_NUM] = [0; STACK_SIZE * CPU_NUM];

#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
extern "C" fn _start() {
    unsafe {
        asm!("\
        mv tp, a0
        add t0, a0, 1
        slli t0, t0, 16
        la sp, {boot_stack}
        add sp, sp, t0
        call main
        ",
        boot_stack = sym STACK,
        options(noreturn)
        );
    }
}

/// rust_main is the entry of the kernel
#[no_mangle]
pub fn main(hart_id: usize, device_tree_addr: usize) -> ! {
    unsafe {
        set_spp(SPP::Supervisor);
    }
    if !STARTED.load(Ordering::Relaxed) {
        clear_bss();
        unsafe {
            println!("{:#x?}", STACK.as_ptr() as usize);
        }
        println!("{}", config::FLAG);
        let machine_info = machine_info_from_dtb(device_tree_addr);
        println!("{:#x?}", machine_info);
        kernel_info(machine_info.memory.end);
        init_print();
        init_memory_system(machine_info.memory.end, true);
        thread_local_init();
        driver::init_dt(device_tree_addr);
        trap::init_trap_subsystem();
        init_per_cpu();
        init_vfs();
        syscall::register_all_syscall();
        task::init_process();
        CPUS.fetch_add(1, Ordering::Release);
        STARTED.store(true, Ordering::Relaxed);
        init_other_hart(hart_id);
    } else {
        while !STARTED.load(Ordering::Relaxed) {
            spin_loop();
        }
        thread_local_init();
        println!("hart {:#x} start", kernel::arch::hart_id());
        init_memory_system(0, false);
        thread_local_init();
        trap::init_trap_subsystem();
        CPUS.fetch_add(1, Ordering::Release);
        loop {}
    }
    timer::set_next_trigger();
    task::schedule::first_into_user();
}

fn init_other_hart(hart_id: usize) {
    for i in 0..CPU_NUM {
        extern "C" {
            fn _start();
        }
        if i != hart_id {
            let res = hart_start(i, _start as usize, 0);
            assert_eq!(res.error, 0);
        }
    }
    // 等待其它cpu核启动
    wait_all_cpu_start();
}

fn wait_all_cpu_start() {
    while CPUS.load(Ordering::Acquire) < CPU_NUM {
        spin_loop()
    }
    println!("all cpu start");
}
