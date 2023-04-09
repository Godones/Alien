#![no_std]
#![no_main]

#[macro_use]
extern crate Mstd;
extern crate alloc;

use alloc::vec::Vec;
use Mstd::fs::{close, open, read, write, OpenFlags, seek};

#[no_mangle]
pub fn main() -> i32 {
    base_read_write_test_fs("/db/dbf1.txt\0");
    base_read_write_test_fs("/fatf1.txt\0");
    0
}


fn base_read_write_test_fs(file:&str){
    const DATA_SIZE: usize = 4096;
    const STR:&[u8] = b"Hello, world!";
    println!("Test basic read/write on file {}", file);
    let data = (0..4096).
        map(|index|{STR[index % STR.len()]})
        .collect::<Vec<u8>>();
    let fd = open(file, OpenFlags::O_CREAT | OpenFlags::O_WRONLY);
    assert!(fd > 0);
    let fd = fd as usize;
    write(fd, &data);
    let mut  read_buf = [0u8; DATA_SIZE];
    let read_len = read(fd, &mut read_buf) as usize;
    assert_eq!(read_len, 0);
    let r = seek(fd, 0, 0);
    assert_eq!(r, 0);
    let read_len = read(fd, &mut read_buf) as usize;
    assert_eq!(read_len, DATA_SIZE);
    assert_eq!(data, read_buf.to_vec());
    close(fd);
    println!("Test basic read/write on file {} passed", file);
}