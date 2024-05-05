#![no_std]
#![no_main]
extern crate alloc;

use alloc::{string::String, vec::Vec};

use Mstd::{fs::get_cwd, println};

#[no_mangle]
fn main(argc: usize, argv: Vec<String>) -> isize {
    println!("argc: {}, argv: {:?}", argc, argv);
    // let file_name = &argv[1];
    // let file = open(file_name,OpenFlags::O_CREAT);
    let mut buf = [0u8; 50];
    let cwd = get_cwd(&mut buf);
    println!("cwd: {}", cwd.unwrap());
    0
}
