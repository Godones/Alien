#![no_std]
#![no_main]

use domain_helper::DomainHelperBuilder;
use Mstd::{
    domain::DomainTypeRaw,
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
    wait_and_update();
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
        if new - start > 30_000 {
            break;
        }
    }
}

fn wait_sec(sec: isize) {
    let now = get_time_ms();
    loop {
        let new = get_time_ms();
        if new - now > sec * 1000 {
            break;
        } else {
            m_yield();
        }
    }
}
fn wait_and_update() {
    wait_sec(10);
    set_priority(0, 0, -16);
    to_new();
    let pid = Mstd::process::getpid();
    let mut now = get_time_ms();
    let start = now;
    loop {
        let new = get_time_ms();
        if new - now > 1000 {
            println!("main pid: {}, time: {}ms", pid, new - start);
            now = new;
        }
        m_yield();
        // 20s -> exit
        if new - start > 5_000 {
            break;
        }
    }
    to_old();
    wait_sec(5);
    to_new();
}

fn to_new() {
    let builder = DomainHelperBuilder::new()
        .domain_name("scheduler")
        .ty(DomainTypeRaw::SchedulerDomain)
        .domain_file_path("/tests/grandom_scheduler\0")
        .domain_file_name("prio_scheduler");
    builder.clone().register_domain_file().unwrap();
    builder.update_domain().unwrap();
    println!("Update scheduler domain to new version success");
}

fn to_old() {
    let builder = DomainHelperBuilder::new()
        .domain_name("scheduler")
        .ty(DomainTypeRaw::SchedulerDomain)
        .domain_file_name("fifo_scheduler");
    builder.update_domain().unwrap();
    println!("Update scheduler domain to old version success");
}
