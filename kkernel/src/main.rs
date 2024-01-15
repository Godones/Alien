#![feature(atomic_from_mut)]
#![feature(ip_in_core)]
#![no_std]
#![no_main]

#[macro_use]
extern crate log;
#[macro_use]
extern crate syscall_table;
#[macro_use]
extern crate platform;
extern crate alloc;

use alloc::boxed::Box;
pub use syscall_table::*;
mod fs;
mod task;
mod time;
mod trap;
mod ipc;
mod mm;
mod net;
mod gui;
mod system;

use core::hint::spin_loop;
use core::panic::PanicInfo;
use core::sync::atomic::{AtomicBool, Ordering};
use platform::{platform_machine_info, system_shutdown};
use crate::task::DriverTaskImpl;

/// 多核启动标志
static STARTED: AtomicBool = AtomicBool::new(false);


#[no_mangle]
fn main(hart_id:usize){
    if STARTED.compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed).is_ok() {
        println!("Boot hart {}", hart_id);
        let machine_info = platform_machine_info();
        println!("{:#?}", machine_info);
        mem::init_memory_system(machine_info.memory.end, true);
        interrupt::init_plic(machine_info.plic.start);
        drivers::register_task_func(Box::new(DriverTaskImpl));
        devices::init_device(Box::new(DriverTaskImpl));
        vfs::init_filesystem().expect("init filesystem failed");
        trap::init_trap_subsystem();
        arch::allow_access_user_memory();
        task::init_process();
        // register all syscall
        syscall_table::init_init_array!();
        STARTED.store(false, Ordering::Relaxed);
    }else {
        while STARTED.load(Ordering::Relaxed) {
            spin_loop();
        }
        mem::init_memory_system(0, false);
        arch::allow_access_user_memory();
        trap::init_trap_subsystem();
        println!("hart {} start", arch::hart_id());
    }
    time::set_next_trigger();
    println!("Begin run task...");
    task::schedule::run_task();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    system_shutdown();
}