#![no_std]
#![no_main]

use Mstd::{
    domain::out_mask,
    process::{exit, fork},
    thread::m_yield,
    time::get_time_ms,
};

#[no_mangle]
fn main() -> isize {
    for _ in 0..2 {
        let pid = fork();
        if pid == 0 {
            test_log();
            exit(0)
        }
        assert!(pid > 0);
    }
    0
}

fn test_log() {
    let mut now = get_time_ms();
    // let pid = Mstd::process::getpid();
    let start = now;
    loop {
        let new = get_time_ms();
        if new - now > 1000 {
            out_mask();
            // println!("pid: {}, time: {}ms", pid, new - start);
            now = new;
        }
        m_yield();
        // 20s -> exit
        if new - start > 20000 {
            break;
        }
    }
}
