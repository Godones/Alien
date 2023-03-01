#![no_std]
#![no_main]

extern crate alloc;

use userlib::println;

#[no_mangle]
fn main() {
    println!("Modular OS Shell");
}
