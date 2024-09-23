#![feature(panic_info_message)]
#![no_std]
#![no_main]

extern crate alloc;
extern crate malloc;
use alloc::boxed::Box;
use core::panic::PanicInfo;

use basic::println;
use corelib::CoreFunction;
use interface::{INTERFACE};
use rref::{domain_id, SharedHeapAlloc};
use storage::StorageArg;


#[no_mangle]
fn main(
    sys: &'static dyn CoreFunction,
    domain_id: u64,
    shared_heap: &'static dyn SharedHeapAlloc,
    storage_arg: StorageArg,
) -> Box<dyn INTERFACE> {
    // init basic
    corelib::init(sys);
    // init rref's shared heap
    rref::init(shared_heap, domain_id);
    basic::logging::init_logger();
    // init storage
    let StorageArg { allocator, storage } = storage_arg;
    storage::init_database(storage);
    storage::init_data_allocator(allocator);
    // activate the domain
    interface::activate_domain();
    // call the real blk driver
    DOMAIN_NAME::main()
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
    basic::backtrace(domain_id());
    loop {}
}
