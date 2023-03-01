#![feature(panic_info_message)]
#![no_std]
#![feature(linkage)]
#![allow(unused)]

use crate::alloc::init_heap;
use crate::process::exit;

mod alloc;
mod fs;
pub mod io;
mod macros;
mod panic;
pub mod process;
mod syscall;
pub mod thread;
pub mod time;

#[no_mangle]
fn _start() -> ! {
    init_heap();
    exit(unsafe { main() as i32 });
}

#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("Cannot find main!");
}
