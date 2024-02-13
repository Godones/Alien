#![no_main]
#![no_std]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use Mstd::fs::mkdir;
use Mstd::println;

#[no_mangle]
fn main(argc: usize, argv: Vec<String>) -> isize {
    println!("argc: {}, argv: {:?}", argc, argv);
    let res = mkdir(&argv[1]);
    if res == -1 {
        println!("mkdir failed");
        return -1;
    } else {
        println!("mkdir success");
    }
    0
}
