#![no_std]
extern crate alloc;

mod context;
mod heap;
mod kthread;
mod stack;
mod cpu;
mod task;
mod schedule;



use crate::fs::read_all;
use crate::schedule::{schedule};
use alloc::sync::Arc;
use alloc::vec::Vec;
use smpscheduler::FifoTask;
use spin::Lazy;
use libsyscall::println;
use timer::get_time_ms;
use crate::cpu::{current_task, GLOBAL_TASK_MANAGER};
use crate::task::Task;


pub fn do_suspend() -> isize {
    let task = current_task().unwrap();
    task.access_inner().update_timer();
    check_task_timer_expired();
    task.update_state(TaskState::Ready);
    schedule();
    0
}

/// 初始进程（0号进程）
pub static INIT_PROCESS: Lazy<Arc<Task>> = Lazy::new(|| {
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
    GLOBAL_TASK_MANAGER.add_task(Arc::new(FifoTask::new(task)));
    println!("Init task success");
}

fn kthread_init() {
    println!("kthread_init start...");
    let mut time = get_time_ms();
    loop {
        let now = get_time_ms();
        if now - time > 1000 {
            // println!("kthread_init tick at {}", now);
            time = now;
        }
        do_suspend();
    }
}

// impl KTask for Task {
//     fn to_wait(&self) {
//         self.update_state(TaskState::Waiting)
//     }
//     fn to_wakeup(&self) {
//         self.update_state(TaskState::Ready)
//     }
//     fn have_signal(&self) -> bool {
//         self.access_inner().signal_receivers.lock().have_signal()
//     }
// }
// pub struct DriverTaskImpl;
// impl KTaskShim for DriverTaskImpl {
//     fn take_current_task(&self) -> Option<Arc<dyn KTask>> {
//         take_current_task().map(|t| t as Arc<dyn KTask>)
//     }
//
//     fn current_task(&self) -> Option<Arc<dyn KTask>> {
//         let task = current_task();
//         task.map(|t| t.clone() as Arc<dyn KTask>)
//     }
//     fn put_task(&self, task: Arc<dyn KTask>) {
//         let task = task.downcast_arc::<Task>().map_err(|_| ()).unwrap();
//         GLOBAL_TASK_MANAGER.add_task(Arc::new(FifoTask::new(task)));
//     }
//     fn suspend(&self) {
//         do_suspend();
//     }
//
//     fn schedule_now(&self, task: Arc<dyn KTask>) {
//         schedule_now(task.downcast_arc::<Task>().map_err(|_| ()).unwrap());
//     }
//     fn transfer_ptr_raw(&self, ptr: usize) -> usize {
//         let task = current_task().unwrap();
//         task.transfer_raw(ptr)
//     }
//     fn transfer_buf_raw(&self, src: usize, size: usize) -> Vec<&mut [u8]> {
//         let task = current_task().unwrap();
//         task.transfer_buffer(src as *const u8, size)
//     }
// }

// online test has no sort.src
// pub static SORT_SRC: &[u8] = include_bytes!("../../../tests/testbin-second-stage/sort.src");
