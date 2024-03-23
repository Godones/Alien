#![feature(panic_info_message)]
#![no_std]
#![feature(linkage)]
#![allow(unused)]
#![allow(non_snake_case)]
#![feature(naked_functions)]
extern crate alloc;

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::arch::asm;

use crate::heap::init_heap;
use crate::process::exit;
use crate::syscall::__system_shutdown;

pub mod common;
pub mod fs;
mod heap;
pub mod io;
pub mod ipc;
mod macros;
pub mod memory;
mod panic;
pub mod process;
pub mod pthread;
pub mod socket;
mod sys;
mod syscall;
pub mod thread;
pub mod time;

#[cfg(feature = "gui")]
pub mod gui;
pub mod sync;

#[no_mangle]
#[naked]
extern "C" fn _start() -> ! {
    unsafe {
        asm!(
            "mv a0,sp
            call _start_rust
            ",
            options(noreturn)
        )
    }
}

#[no_mangle]
fn _start_rust(argc_ptr: usize) {
    let argc = unsafe { (argc_ptr as *const usize).read_volatile() };
    let argv = argc_ptr + core::mem::size_of::<usize>();
    init_heap();

    let argv = parse_args(argc, argv); //todo!(env)
    exit(unsafe { main(argc, argv) });
}

fn parse_args(argc: usize, argv: usize) -> Vec<String> {
    let mut args = Vec::new();
    for i in 0..argc {
        let arg = unsafe { *(argv as *const *const u8).add(i) };
        let len = unsafe { common::strlen(arg) };
        let arg = unsafe {
            let slice = core::slice::from_raw_parts(arg, len);
            core::str::from_utf8_unchecked(slice)
        };
        args.push(arg.to_string());
    }
    args
}

#[linkage = "weak"]
#[no_mangle]
fn main(argc: usize, argv: Vec<String>) -> i32 {
    panic!("Cannot find main!");
}

pub fn system_shutdown() -> ! {
    __system_shutdown();
    panic!("Shutdown failed!");
}
