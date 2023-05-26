#![no_main]
#![no_std]


extern crate alloc;

use alloc::vec;

use rand::{RngCore, SeedableRng};
use rand::rngs::SmallRng;

use Mstd::fs::{close, open, OpenFlags, read, seek, write};
use Mstd::println;
use Mstd::time::get_time_ms;

const FILE_SIZE: usize = 1024 * 1024 * 16;
//16MB
const BLOCK_SIZE: usize = 1024 * 1 * 1024;

const ITER: usize = 1000;

fn rand_read_write_test(path: &str) {
    let time = get_time_ms();
    let mut small_rng = SmallRng::seed_from_u64(time as u64);
    let fd = open(path, OpenFlags::O_RDWR | OpenFlags::O_CREAT);
    assert!(fd > 0);
    let mut buf = vec![0u8; BLOCK_SIZE];
    for _ in 0..FILE_SIZE / BLOCK_SIZE {
        let w = write(fd as usize, &buf);
        assert_eq!(w as usize, BLOCK_SIZE);
    }
    close(fd as usize);

    let fd = open(path, OpenFlags::O_RDONLY);
    assert!(fd > 0);
    println!("fd: {}", fd);
    let start = get_time_ms();
    let mut count = 0;
    for _ in 0..ITER {
        let offset = small_rng.next_u64();
        let offset = offset % (FILE_SIZE as u64 - BLOCK_SIZE as u64);
        let res = seek(fd as usize, offset as isize, 0);
        assert_ne!(res, -1);
        let read = read(fd as usize, &mut buf);
        assert_ne!(read, -1);
        assert_eq!(read as usize, BLOCK_SIZE);
        count += read;
    }
    let end = get_time_ms();
    let time = end - start;
    let ops = ITER as f64 * 1000 as f64 / time as f64;
    let throughput = count as f64 * 1000 as f64 / 1024 as f64 / time as f64 / 1024 as f64;
    println!("Random read test:");
    println!("Elapsed time: {}ms", time);
    println!("Read: {}MB", count / 1024 / 1024);
    println!("Throughput: {}MB/s", throughput);
    println!("Operations: {}ops/s", ops);

    // rand write
    let start = get_time_ms();
    let mut count = 0;
    for _ in 0..ITER {
        let offset = small_rng.next_u64();
        let offset = offset % (FILE_SIZE as u64 - BLOCK_SIZE as u64);
        let res = seek(fd as usize, offset as isize, 0);
        assert_ne!(res, -1);
        let w = write(fd as usize, &buf);
        assert_ne!(w, -1);
        count += w;
    }
    println!("write over");
    let end = get_time_ms();
    let time = end - start;
    let ops = ITER as f64 * 1000 as f64 / time as f64;
    let throughput = count as f64 * 1000 as f64 / 1024 as f64 / time as f64 / 1024 as f64;
    println!("Random write test:");
    println!("Elapsed time: {}ms", time);
    println!("Write: {}MB", count / 1024 / 1024);
    println!("Throughput: {}MB/s", throughput);
    println!("Operations: {}ops/s", ops);
    close(fd as usize);
}

#[no_mangle]
fn main() {
    println!("Rand test...");
    rand_read_write_test("/fatrandrwtest\0");
    rand_read_write_test("/db/dbrandrwtest\0");
}