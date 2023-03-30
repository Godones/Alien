#![no_std]
#![no_main]

extern crate alloc;

use Mstd::fs::{close, open, OpenFlags, write};
use Mstd::println;

#[no_mangle]
fn main() -> isize {
    println!("try write to db .......");
    let fd  = open("/db/dbf1.txt\0", OpenFlags::O_WRONLY|OpenFlags::O_CREAT);
    assert!(fd > 0);
    let buf = b"hello world";
    write(fd as usize, buf);
    close(fd as usize);
    0
}