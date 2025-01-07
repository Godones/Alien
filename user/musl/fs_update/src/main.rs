use std::{
    env,
    fs::File,
    io::{Read, Seek, SeekFrom},
    thread::yield_now,
    time::Instant,
};

use domain_helper::DomainHelperBuilder;
use domain_info::DomainTypeRaw;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <block size>", args[0]);
        return;
    }
    let blk_size = args[1].parse::<usize>().unwrap();
    read_bash_test(blk_size);

    let read_thread = std::thread::spawn(|| {
        read_bash_for_sec(10);
    });
    let update_thread = std::thread::spawn(|| {
        update_vfs();
    });

    read_thread.join().unwrap();
    update_thread.join().unwrap();
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

fn read_bash_for_sec(sec: usize) {
    let mut file = File::open("/tests/bash").unwrap();
    let now = Instant::now();
    let mut start = now;
    let mut buf = vec![0u8; 4096];

    // record bytes per 100ms
    let records = sec * 1000 / 100;
    let mut bytes_per_100ms = Vec::with_capacity(records);

    let mut bytes = 0;
    let mut record_bytes = 0;
    loop {
        let res = file.read(&mut buf).unwrap();
        if res == 0 {
            // rewind
            file.seek(SeekFrom::Start(0)).unwrap();
        }
        bytes += res;
        let new_now = Instant::now();
        if new_now.duration_since(start).as_millis() >= 100 {
            let read_bytes = bytes - record_bytes;
            bytes_per_100ms.push((read_bytes, new_now.duration_since(now).as_millis()));
            record_bytes = bytes;
            // reset start
            start = new_now;
        }
        if new_now.duration_since(now).as_secs() >= sec as u64 {
            break;
        }
    }
    for i in 0..bytes_per_100ms.len() {
        println!("{}ms: {} bytes", bytes_per_100ms[i].1, bytes_per_100ms[i].0);
    }
    let ms = now.elapsed().as_millis();
    let speed = bytes as f64 * 1000.0 / ms as f64 / 1024.0;
    println!(
        "Read {} bytes in {}ms, speed: {} KB/s",
        bytes, ms, speed as isize
    );
}

fn update_vfs() {
    let now = Instant::now();
    loop {
        let elapsed = now.elapsed().as_secs();
        if elapsed >= 5 {
            break;
        } else {
            yield_now()
        }
    }

    let builder = DomainHelperBuilder::new()
        .ty(DomainTypeRaw::VfsDomain)
        .domain_file_path("/tests/gvfs2\0")
        .domain_file_name("vfs2")
        .domain_name("vfs");

    builder.clone().register_domain_file().unwrap();

    builder.update_domain().unwrap();
    println!("Test register and update vfs domain success");
}
