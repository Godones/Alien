#![no_main]
#![no_std]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use Mstd::fs::{FileMode, openat, OpenFlags};
use Mstd::println;

#[no_mangle]
fn main(_argc: usize, argv: Vec<String>) -> isize {
    let file = &argv[1];
    let r = openat(0, file, OpenFlags::O_CREAT|OpenFlags::O_RDWR,FileMode::FMODE_RDWR);
    if r < 0 {
        println!("touch {} failed", file);
        return -1;
    } else {
        println!("touch {} success", file);
    }
    0
}
