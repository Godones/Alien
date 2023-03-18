#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use Mstd::fs::{close, open, OpenFlags, read};
use Mstd::println;

#[no_mangle]
fn main(_argc:usize, argv: Vec<String>) -> isize {
    let file_name = &argv[1];
    let fd= open(file_name,OpenFlags::O_RDONLY);
    if fd != -1{
        let mut buf = [0u8; 1024];
        while let len = read(fd as usize, &mut buf) {
            if len==0{
                break
            }
            println!("{}",core::str::from_utf8(&buf[..len as usize]).unwrap());
        }
        close(fd as usize);
    }
    0
}
