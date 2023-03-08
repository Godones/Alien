#![feature(panic_info_message)]
#![no_std]
#![feature(linkage)]
#![allow(unused)]

extern crate alloc;
use crate::heap::init_heap;
use crate::process::exit;
use crate::syscall::sys_shutdown;

mod heap;
pub mod fs;

mod macros;
mod panic;
pub mod process;
mod syscall;
pub mod thread;
pub mod time;
pub mod io;
mod sys;

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

pub fn shutdown() -> ! {
    sys_shutdown();
    panic!("Shutdown failed!");
}