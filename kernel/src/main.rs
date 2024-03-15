#![feature(panic_info_message)]
#![no_std]
#![no_main]
mod panic;

#[macro_use]
extern crate platform;
extern crate alloc;
mod domain;

use core::hint::spin_loop;
use core::sync::atomic::{AtomicBool, Ordering};

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
        // interrupt::init_plic(machine_info.plic.start);
        // drivers::register_task_func(Box::new(DriverTaskImpl));
        // devices::init_device(Box::new(DriverTaskImpl));
        // vfs::init_filesystem().expect("init filesystem failed");
        kcore::trap::init_trap_subsystem();
        arch::allow_access_user_memory();
        // task::init_task();
        // register all syscall
        // syscall_table::init_init_array!();
        domain::load_domains();

        STARTED.store(false, Ordering::Relaxed);
    } else {
        while STARTED.load(Ordering::Relaxed) {
            spin_loop();
        }
        mem::init_memory_system(0, false);
        arch::allow_access_user_memory();
        kcore::trap::init_trap_subsystem();
        println!("hart {} start", arch::hart_id());
    }
    timer::set_next_trigger();
    println!("Begin run task...");
    kcore::run_task();
    platform::system_shutdown();
}
