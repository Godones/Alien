#![feature(atomic_from_mut)]
// To handle compiler bugs
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![no_std]
#![no_main]

#[macro_use]
extern crate log;
#[macro_use]
extern crate syscall_table;
#[macro_use]
extern crate platform;
extern crate alloc;
extern crate unwinder;
use alloc::boxed::Box;

pub use syscall_table::*;
mod fs;
mod gui;
mod ipc;
mod mm;
mod net;
mod system;
mod task;
mod time;
mod trap;

use core::{
    hint::spin_loop,
    sync::atomic::{AtomicBool, Ordering},
};

use platform::platform_machine_info;

use crate::task::DriverTaskImpl;

/// 多核启动标志
static STARTED: AtomicBool = AtomicBool::new(false);

#[no_mangle]
fn main(hart_id: usize) {
    if STARTED
        .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
        .is_ok()
    {
        println!("Boot hart {}", hart_id);
        let machine_info = platform_machine_info();
        println!("{:#?}", machine_info);
        mem::init_memory_system(machine_info.memory.end, true);
        interrupt::init_plic(machine_info.plic.start);
        shim::register_task_func(Box::new(DriverTaskImpl));
        devices::init_device();
        vfs::init_filesystem().expect("init filesystem failed");
        trap::init_trap_subsystem();
        arch::allow_access_user_memory();
        task::init_task();
        // register all syscall
        syscall_table::init_init_array!();
        STARTED.store(false, Ordering::Relaxed);
    } else {
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
