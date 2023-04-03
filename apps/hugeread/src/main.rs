#![no_std]
#![no_main]

#[macro_use]
extern crate Mstd;

use Mstd::fs::{close, open, read, write, OpenFlags};
use Mstd::time::get_time_ms;

#[no_mangle]
pub fn main() -> i32 {
    write_fs("FAT32", "f1.txt\0");
    write_fs("DBFS", "/db/f1.txt\0");
    test_read_fs("FAT32", "f1.txt\0");
    test_read_fs("DBFS", "/db/f1.txt\0");
    0
}

fn write_fs(name: &str, path: &str) {
    let mut buffer = [0u8; 1024]; // 1KiB
    for i in 0..buffer.len() {
        buffer[i] = i as u8;
    }
    let f = open(path, OpenFlags::O_CREAT | OpenFlags::O_WRONLY);
    if f < 0 {
        panic!("Open test file failed!");
    }
    let f = f as usize;
    let size_mb = 1usize;
    let mut count = 0;
    for _ in 0..1024 * size_mb {
        let len = write(f, &buffer);
        if len as usize != buffer.len() {
            println!("len = {}", len);
            panic!("Write test file failed!");
        }
        count += len as usize;
    }
    close(f);
    println!("{} write {} bytes", name, count);
}

fn test_read_fs(name: &str, path: &str) {
    println!("{} read {}MiB", name, 1);
    let mut buffer = [0u8; 1024]; // 1KiB
    let f = open(path, OpenFlags::O_RDWR);
    if f < 0 {
        panic!("Open test file failed!");
    }
    let f = f as usize;
    let start = get_time_ms();
    let size_mb = 1usize;
    let mut count = 0;
    for _ in 0..1024 * size_mb {
        let len = read(f, &mut buffer);
        if len as usize != buffer.len() {
            println!("count :{} len = {}", count, len);
            panic!("Read test file failed!");
        }
        count += len as usize;
    }
    close(f);
    let time_ms = (get_time_ms() - start) as usize;
    println!("read {} bytes", count);
    let speed_kbs = size_mb * 1000000 / time_ms;
    println!("time cost = {}ms, read speed = {}KiB/s", time_ms, speed_kbs);
}
