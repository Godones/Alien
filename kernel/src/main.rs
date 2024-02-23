#![feature(panic_info_message)]
#![no_std]
#![no_main]
mod panic;

#[macro_use]
extern crate platform;

#[no_mangle]
fn main(hart_id: usize) -> ! {
    println!("boot hart id: {}", hart_id);
    let machine_info = platform::platform_machine_info();
    println!("{:#x?}", machine_info);
    mem::init_memory_system(machine_info.memory.end, true);
    domain_loader::test_domain();
    println!("shutdown");
    platform::system_shutdown()
}
