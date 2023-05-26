#![no_std]
#![no_main]

#[macro_use]
extern crate Mstd;
extern crate alloc;

use alloc::vec;

use Mstd::fs::{close, open, OpenFlags, write};
use Mstd::time::get_time_ms;

const DATA_SIZE: usize = 1024 * 1024 * 10;
//10MB
const BUF_SIZE: usize = 1024 * 1024;

#[no_mangle]
pub fn main() -> i32 {
    test_for_fs("FAT32", "fatseqwrite.txt\0");
    test_for_fs("DBFS", "/db/seqwrite.txt\0");
    0
}

fn test_for_fs(name: &str, path: &str) {
    println!("{} write {}MiB", name, 10);

    let mut buffer = vec![0u8; BUF_SIZE]; // 1KiB
    for i in 0..buffer.len() {
        buffer[i] = i as u8;
    }
    let start = get_time_ms();
    let f = open(path, OpenFlags::O_CREAT | OpenFlags::O_WRONLY);
    if f < 0 {
        panic!("Open test file failed!");
    }
    println!("file open cost {}ms", get_time_ms() - start);

    let f = f as usize;
    let start = get_time_ms();
    let mut count = 0;
    for _ in 0..DATA_SIZE / BUF_SIZE {
        let len = write(f, &buffer);
        if len as usize != buffer.len() {
            println!("count :{} len = {}", count, len);
            panic!("Write test file failed!");
        }
        count += len as usize;
    }
    close(f);
    let time_ms = (get_time_ms() - start) as usize;
    println!("write {} bytes", count);
    assert_eq!(count, DATA_SIZE);
    let speed = 10.0 / (time_ms as f64 / 1000.0);
    println!("time cost = {}ms, write speed = {}MB/s", time_ms, speed);
}
