#![no_std]
#![feature(panic_info_message)]

extern crate alloc;

use alloc::boxed::Box;
use alloc::sync::Arc;
use core::panic::PanicInfo;
use interface::Uart;
use libsyscall::Syscall;
use rref::SharedHeap;

#[no_mangle]
fn main(sys: Box<dyn Syscall>, domain_id: u64, shared_heap: Box<dyn SharedHeap>) -> Arc<dyn Uart> {
    // init rref's shared heap
    rref::init(shared_heap, domain_id);
    // init libsyscall
    libsyscall::init(sys);
    // activate the domain
    interface::activate_domain();
    // call the real uart driver
    // uart::main()
    unimplemented!()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(p) = info.location() {
        libsyscall::println!(
            "line {}, file {}: {}",
            p.line(),
            p.file(),
            info.message().unwrap()
        );
    } else {
        libsyscall::println!("no location information available");
    }
    interface::deactivate_domain();
    libsyscall::backtrace();
    loop {}
}
