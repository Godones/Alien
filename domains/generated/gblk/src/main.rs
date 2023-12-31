#![feature(panic_info_message)]
#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
use core::panic::PanicInfo;
use interface::BlkDevice;
use libsyscall::{println, Syscall};
use rref::SharedHeap;

#[no_mangle]
fn main(
    sys: Box<dyn Syscall>,
    domain_id: u64,
    shared_heap: Box<dyn SharedHeap>,
    virtio_blk_addr: usize,
) -> Box<dyn BlkDevice> {
    // init libsyscall
    libsyscall::init(sys, domain_id);
    // init rref's shared heap
    rref::init(shared_heap);
    // activate the domain
    interface::activate_domain();
    // call the real blk driver
    blk::main(virtio_blk_addr)
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(p) = info.location() {
        println!(
            "line {}, file {}: {}",
            p.line(),
            p.file(),
            info.message().unwrap()
        );
    } else {
        println!("no location information available");
    }
    interface::deactivate_domain();
    libsyscall::backtrace();
    loop {}
}
