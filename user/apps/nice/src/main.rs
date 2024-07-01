#![no_std]
#![no_main]

use Mstd::{
    println,
    process::{exit, fork, get_priority, set_priority},
    thread::m_yield,
    time::get_time_ms,
};
const MAX_COUNT: i32 = 8;

#[no_mangle]
fn main() -> isize {
    let prio = -2;
    for i in 0..MAX_COUNT {
        let pid = fork();
        if pid == 0 {
            test_log(prio - i * 2);
            exit(0)
        }
        assert!(pid > 0);
    }
    0
}

fn test_log(prio: i32) {
    set_priority(0, 0, prio);
    let prio = get_priority(0, 0);
    let pid = Mstd::process::getpid();
    println!("pid: {}, prio: {}", pid, prio);
    let mut now = get_time_ms();
    let start = now;
    loop {
        let new = get_time_ms();
        if new - now > 1000 {
            println!("pid: {}, time: {}ms", pid, new - start);
            now = new;
        }
        m_yield();
        // 20s -> exit
        if new - start > 10000 {
            break;
        }
    }
}
