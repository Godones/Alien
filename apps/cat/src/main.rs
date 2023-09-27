#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use Mstd::fs::{close, open, read, OpenFlags};
use Mstd::{print, println};

#[no_mangle]
fn main(_argc: usize, argv: Vec<String>) -> isize {
    let file_name = &argv[1];
    let file_name = if !file_name.ends_with('\0') {
        file_name.clone() + "\0"
    } else {
        file_name.clone()
    };
    let fd = open(&file_name, OpenFlags::O_RDONLY);
    if fd > 0 {
        let mut buf = [0u8; 10];
        loop {
            let len = read(fd as usize, &mut buf);
            if len <= 0 {
                if len < 0 {
                    println!("read error");
                }
                break;
            }
            print!("{}", core::str::from_utf8(&buf[..len as usize]).unwrap());
        }
        println!();
        close(fd as usize);
    }
    0
}
