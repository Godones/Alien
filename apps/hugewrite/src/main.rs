#![no_std]
#![no_main]

#[macro_use]
extern crate Mstd;

use Mstd::fs::{close, open, write, OpenFlags};
use Mstd::time::get_time_ms;

#[no_mangle]
pub fn main() -> i32 {
    test_for_fs("FAT32", "f1.txt\0");
    test_for_fs("DBFS", "/db/f1.txt\0");
    0
}

fn test_for_fs(name: &str, path: &str) {
    println!("{} write {}MiB", name, 1);

    let mut buffer = [0u8; 1024]; // 1KiB
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
    let size_mb = 1usize;
    let mut count = 0;
    for _ in 0..1024 * size_mb {
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
    let speed_kbs = count / time_ms;
    println!("time cost = {}ms, write speed = {}KB/s", time_ms, speed_kbs);
}
