#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use Mstd::fs::list;
#[no_mangle]
fn main(_: usize, argv: Vec<String>) -> isize {
    if argv.len() == 1 {
        list("./")
    } else {
        list(&argv[1])
    }
}
