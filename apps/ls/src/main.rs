#![no_std]
#![no_main]

#[no_mangle]
fn main() {
    Mstd::fs::list();
}