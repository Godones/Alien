use std::{env, fs::File, io::Read, time::Instant};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <block size>", args[0]);
        return;
    }
    let blk_size = args[1].parse::<usize>().unwrap();
    read_bash_test(blk_size);
    // in cache
    read_bash_test(blk_size);
    // read_once_test();
}

fn read_bash_test(blk_size: usize) {
    let mut file = File::open("/tests/bash").unwrap();
    let now = Instant::now();
    let mut buf = vec![0u8; blk_size];
    let mut bytes = 0;
    loop {
        let res = file.read(&mut buf).unwrap();
        if res == 0 {
            break;
        }
        bytes += res;
    }
    let ms = now.elapsed().as_millis();
    let speed = bytes as f64 * 1000.0 / ms as f64 / 1024.0;
    println!(
        "Read {} bytes in {}ms, speed: {} KB/s",
        bytes, ms, speed as isize
    );
}

#[allow(unused)]
fn read_once_test() {
    let mut file = File::open("/tests/bash").unwrap();
    let now = Instant::now();
    let mut buf = [0u8; 4096];
    let res = file.read(&mut buf).unwrap();
    let ms = now.elapsed().as_micros();
    println!("Read {} bytes in {}us", res, ms);
}
