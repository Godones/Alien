#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
use core::panic::PanicInfo;
use interface::BlkDevice;
use libsyscall::Syscall;

#[no_mangle]
fn main(sys: Box<dyn Syscall>, domain_id: u64) -> Box<dyn BlkDevice> {
    // init libsyscall
    libsyscall::init(sys, domain_id);
    libsyscall::write_console("Blk domain start");
    // call the real blk driver
    blk::main()
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}