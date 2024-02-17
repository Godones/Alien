//! Alien 中有关进程管理的相关数据结构
//!
//! [`context`] 子模块定义了 Alien 中线程上下文的相关结构.
//! [`cpu`] 子模块中指明了 Alien 中有关进程的系统调用 和 多核的相关支持。
//! [`heap`] 子模块定义了 Alien 记录进程堆空间的相关信息的结构。
//! [`schedule`] 子模块指明了 Alien 中有关 CPU 调度的相关机制
//! [`stack`] 子模块定义了 Alien 中有关内核栈的相关结构。
//! [`task`] 子模块定义了 Alien 中有关进程控制块的定义。
use crate::fs::read_all;
pub use crate::task::task::FsContext;
use alloc::sync::Arc;
use alloc::vec::Vec;
pub use cpu::*;
use devices::DeviceWithTask;
use drivers::{DriverTask, DriverWithTask};
use smpscheduler::FifoTask;
use spin::Lazy;
pub use task::{StatisticalData, Task, TaskState};
use timer::get_time_ms;

mod context;
mod cpu;
mod heap;
mod kthread;
pub mod schedule;
mod stack;
mod task;

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
    kthread::ktread_create(kthread_test, "kthread_test").unwrap();
    let task = INIT_PROCESS.clone();
    GLOBAL_TASK_MANAGER.add_task(Arc::new(FifoTask::new(task)));
    println!("Init process success");
}

fn kthread_test() {
    let mut time = get_time_ms();
    println!("kthread_test start ...",);
    loop {
        let now = get_time_ms();
        if now - time > 1000 {
            // println!("kthread_test tick at {}", now);
            time = now;
        }
        do_suspend();
    }
}

impl DriverTask for Task {
    fn to_wait(&self) {
        self.update_state(TaskState::Waiting)
    }

    fn to_wakeup(&self) {
        self.update_state(TaskState::Ready)
    }

    fn have_signal(&self) -> bool {
        self.access_inner().signal_receivers.lock().have_signal()
    }
}
pub struct DriverTaskImpl;
impl DriverWithTask for DriverTaskImpl {
    fn get_task(&self) -> Arc<dyn DriverTask> {
        let task = current_task().unwrap();
        task.clone()
    }

    fn put_task(&self, task: Arc<dyn DriverTask>) {
        let task = task.downcast_arc::<Task>().map_err(|_| ()).unwrap();
        GLOBAL_TASK_MANAGER.add_task(Arc::new(FifoTask::new(task)));
    }

    fn suspend(&self) {
        do_suspend();
    }
}

impl DeviceWithTask for DriverTaskImpl {
    fn transfer_ptr_raw(&self, ptr: usize) -> usize {
        let task = current_task().unwrap();
        task.transfer_raw(ptr)
    }

    fn transfer_buf_raw(&self, src: usize, size: usize) -> Vec<&mut [u8]> {
        let task = current_task().unwrap();
        task.transfer_buffer(src as *const u8, size)
    }
}

// online test has no sort.src
// pub static SORT_SRC: &[u8] = include_bytes!("../../../sdcard/sort.src");
