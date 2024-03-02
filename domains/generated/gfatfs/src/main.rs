#![feature(panic_info_message)]
#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
use alloc::sync::Arc;
use core::panic::PanicInfo;
use interface::FsDomain;
use libsyscall::{println, Syscall};
use rref::SharedHeap;

#[no_mangle]
fn main(
    sys: Box<dyn Syscall>,
    domain_id: u64,
    shared_heap: Box<dyn SharedHeap>,
) -> Arc<dyn FsDomain> {
    rref::init(shared_heap, domain_id);
    // init libsyscall
    libsyscall::init(sys);
    // activate the domain
    interface::activate_domain();
    // call the real fatfs
    // let blk_device = blk_device;
    // let res = blk_device.read(0, rref::RRef::new([0; 512]));
    // println!("read res is err: {:?}?", res.err());
    fatfs::main()
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
    // deactivate the domain
    interface::deactivate_domain();
    libsyscall::backtrace();
    loop {}
}
