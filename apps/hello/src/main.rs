#![no_std]
#![no_main]


use userlib::println;

#[no_mangle]
fn main() {
    println!("Hello, world!");
}
