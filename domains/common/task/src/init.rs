use alloc::{sync::Arc, vec::Vec};

use basic::println;
use spin::Lazy;

use crate::{kthread, processor::add_task, scheduler_domain, task::Task, vfs_shim::read_all};

static INIT_PROCESS: Lazy<Arc<Task>> = Lazy::new(|| {
    let mut data = Vec::new();
    read_all("/tests/init", &mut data);
    assert!(data.len() > 0);
    let task = Task::from_elf("/tests/init", data.as_slice()).unwrap();
    Arc::new(task)
});

/// 将初始进程加入进程池中进行调度
pub fn init_task() {
    kthread::ktread_create(kthread_init, "kthread_test").unwrap();
    let task = INIT_PROCESS.clone();
    add_task(task);
}

fn kthread_init() {
    println!("kthread_init start...");
    let mut time = basic::time::read_time_ms();
    loop {
        let now = basic::time::read_time_ms();
        if now - time > 1000 {
            // println!("kthread_init tick at {}", now);
            time = now;
        }
        scheduler_domain!().yield_now().unwrap();
    }
    // kthread::ktrhead_exit();
}
