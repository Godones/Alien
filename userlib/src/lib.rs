#![feature(panic_info_message)]
#![no_std]
#![feature(linkage)]
#![allow(unused)]

extern crate alloc;
use crate::heap::init_heap;
use crate::process::exit;
use crate::syscall::sys_shutdown;

pub mod fs;
mod heap;

pub mod io;
mod macros;
mod panic;
pub mod process;
mod sys;
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

pub fn shutdown() -> ! {
    sys_shutdown();
    panic!("Shutdown failed!");
}
