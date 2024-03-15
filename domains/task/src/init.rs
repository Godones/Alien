use crate::kthread;
use crate::processor::add_task;
use crate::scheduler::do_suspend;
use crate::task::Task;
use crate::vfs_shim::read_all;
use alloc::sync::Arc;
use alloc::vec::Vec;
use libsyscall::println;
use spin::Lazy;

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
    println!("Init task domain success");
}

fn kthread_init() {
    println!("kthread_init start...");
    let mut time = libsyscall::get_time_ms();
    loop {
        let now = libsyscall::get_time_ms();
        if now - time > 1000 {
            println!("kthread_init tick at {}", now);
            time = now;
        }
        do_suspend();
    }
}
