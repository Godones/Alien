#![no_std]
#![no_main]
extern crate  alloc;

use alloc::string::String;
use alloc::vec::Vec;
use Mstd::fs::{open, OpenFlags};

#[no_mangle]
fn main(argc:usize,argv:Vec<String>) -> isize {
    assert_eq!(argc, 2);
    let file_name = &argv[1];
    // let file = open(file_name,OpenFlags::O_CREAT);
    0
}