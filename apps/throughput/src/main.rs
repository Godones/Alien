#![no_std]
#![no_main]
//! Test filesystem of throughput
use Mstd::fs::{close, open, read, write, OpenFlags};
use Mstd::println;
use Mstd::time::{get_time_ms, get_time_of_day, TimeVal};

const FILE_SIZE: usize = 1024 * 1024 * 10;

#[cfg(feature = "8k")]
const BLOCK_SIZE: usize = 8192;
#[cfg(feature = "4k")]
const BLOCK_SIZE: usize = 4096;
#[cfg(feature = "1k")]
const BLOCK_SIZE: usize = 1024;
#[cfg(feature = "512")]
const BLOCK_SIZE: usize = 512;
#[cfg(feature = "256")]
const BLOCK_SIZE: usize = 256;

// generate a file with FILE_SIZE

fn generate_file(path: &str) {
    let mut file = open(path, OpenFlags::O_RDWR | OpenFlags::O_CREAT);
    assert_ne!(file, -1);
    let mut buf = [0u8; BLOCK_SIZE];
    // fill data
    for i in 0..BLOCK_SIZE {
        buf[i] = get_time_ms() as u8;
    }
    for _ in 0..FILE_SIZE / BLOCK_SIZE {
        let len = write(file as usize, &buf);
        assert_eq!(len as usize, BLOCK_SIZE);
    }
    close(file as usize);
}

fn test_throughput(path: &str) {
    let mut file = open(path, OpenFlags::O_RDONLY);
    assert_ne!(file, -1);
    let mut buf = [0u8; BLOCK_SIZE];
    let mut total = 0;
    let start = get_time_ms();
    assert!(start > 0);
    loop {
        let len = read(file as usize, &mut buf);
        if len == 0 {
            break;
        }
        total += len;
    }
    assert_eq!(total as usize, FILE_SIZE);
    let end = get_time_ms();
    assert!(end > 0);
    close(file as usize);
    let time_ms = end - start;
    let speed_kbs = total / time_ms;
    println!("time cost = {}ms, read speed = {}KB/s", time_ms, speed_kbs);
}

fn fat32() {
    generate_file("/fattest\0");
    test_throughput("/fattest\0");
}

fn dbfs() {
    generate_file("/db/dbtest\0");
    test_throughput("/db/dbtest\0");
}

#[no_mangle]
fn main() {
    fat32();
    dbfs();
}
