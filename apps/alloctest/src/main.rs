#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;

use Mstd::println;

#[no_mangle]
fn main() {
    println!("Alloc test!");
    let mut v = Vec::new();
    for i in 0..2000 {
        v.push(i);
    }
    v.clear();
    println!("Alloc test success!");
}
