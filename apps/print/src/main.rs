#![no_std]
#![no_main]

use Mstd::println;

#[no_mangle]
fn main() {
    let str = Mstd::io::read_line();
    println!("{}", str);
}
