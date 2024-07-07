#![no_std]
#![no_main]
use Mstd::{
    fs::{close, open, read, OpenFlags},
    println,
    time::get_time_ms,
};

#[no_mangle]
fn main() -> isize {
    read_bash_test();
    // in cache
    read_bash_test();
    0
}

fn read_bash_test() -> isize {
    let bash_file_test = open("/tests/bash\0", OpenFlags::O_RDONLY);
    if bash_file_test < 0 {
        println!("Failed to open /tests/bash");
        return -1;
    }
    let now = get_time_ms();
    let mut buf = [0u8; 100];
    let mut bytes = 0;
    loop {
        let res = read(bash_file_test as usize, &mut buf);
        if res == 0 {
            break;
        }
        bytes += res;
    }
    let new = get_time_ms();
    let time = new - now;
    let speed = bytes as f64 / time as f64 * 1000.0 / 1024.0;
    println!(
        "Read {} bytes in {}ms, speed: {} KB/s",
        bytes, time, speed as isize
    );
    close(bash_file_test as _);
    0
}
