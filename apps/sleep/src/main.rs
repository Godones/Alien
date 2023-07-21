#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use Mstd::time::sleep;

#[no_mangle]
fn main(_argc: usize, argv: Vec<String>) {
    let sleep_time = &argv[1];
    let sleep_time: usize = sleep_time.parse().unwrap();
    sleep(sleep_time * 1000);
}
