#![feature(panic_info_message)]
#![feature(naked_functions)]
#![feature(box_into_inner)]
#![feature(trait_upcasting)]
#![allow(unused_unsafe)]
#![no_std]
#![no_main]
mod panic;

#[macro_use]
extern crate platform;
#[macro_use]
extern crate log;
extern crate alloc;
mod bus;
mod domain;
mod domain_helper;
mod domain_loader;
mod domain_proxy;
mod error;
mod sync;
mod task;
mod timer;
mod trap;

use core::{
    hint::spin_loop,
    sync::atomic::{AtomicBool, Ordering},
};

/// 多核启动标志
static STARTED: AtomicBool = AtomicBool::new(false);

#[no_mangle]
fn main(hart_id: usize) {
    if STARTED
        .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
        .is_ok()
    {
        println!("Boot hart {}", hart_id);
        let machine_info = platform::platform_machine_info();
        println!("{:#?}", machine_info);
        mem::init_memory_system(machine_info.memory.end, true);
        trap::init_trap_subsystem();
        arch::allow_access_user_memory();
        bus::init_with_dtb().unwrap();
        domain::load_domains().unwrap();
        STARTED.store(false, Ordering::Relaxed);
    } else {
        while STARTED.load(Ordering::Relaxed) {
            spin_loop();
        }
        mem::init_memory_system(0, false);
        arch::allow_access_user_memory();
        trap::init_trap_subsystem();
        println!("hart {} start...", arch::hart_id());
    }
    timer::set_next_trigger();
    println!("Begin run task...");
    task::run_task();
    platform::system_shutdown();
}
