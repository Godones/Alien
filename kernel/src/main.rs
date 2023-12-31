#![feature(panic_info_message)]
#![no_std]
#![no_main]
mod info;
mod panic;

#[macro_use]
extern crate platform;

use basemachine::machine_info_from_dtb;
use info::kernel_info;

#[no_mangle]
fn main(hart_id: usize, dtree: usize) -> ! {
    println!("{}", config::ALIEN_FLAG);
    println!("boot hart id: {}, device tree addr: {:#x}", hart_id, dtree);
    let machine_info = machine_info_from_dtb(dtree);
    let memory_start = kernel_info(machine_info.memory.end);
    println!("{:#x?}", machine_info);
    platform::platform_init();
    mem::init_memory_system(memory_start, machine_info.memory.end, true);
    domain_loader::test_domain();
    println!("shutdown");
    platform::system_shutdown()
}
