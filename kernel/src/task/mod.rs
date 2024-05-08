//! Alien 中有关进程管理的相关数据结构
//!
//! [`context`] 子模块定义了 Alien 中线程上下文的相关结构.
//! [`cpu`] 子模块中指明了 Alien 中有关进程的系统调用 和 多核的相关支持。
//! [`heap`] 子模块定义了 Alien 记录进程堆空间的相关信息的结构。
//! [`schedule`] 子模块指明了 Alien 中有关 CPU 调度的相关机制
//! [`stack`] 子模块定义了 Alien 中有关内核栈的相关结构。
//! [`task`] 子模块定义了 Alien 中有关进程控制块的定义。
use alloc::{sync::Arc, vec::Vec};

pub use cpu::*;
use shim::{KTask, KTaskShim};
use smpscheduler::FifoTask;
use spin::Lazy;
pub use task::{StatisticalData, Task, TaskState};
use timer::get_time_ms;

pub use crate::task::task::FsContext;
use crate::{fs::read_all, task::schedule::schedule_now};

mod context;
mod cpu;
mod kthread;
mod resource;
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

impl KTask for Task {
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
impl KTaskShim for DriverTaskImpl {
    fn take_current_task(&self) -> Option<Arc<dyn KTask>> {
        take_current_task().map(|t| t as Arc<dyn KTask>)
    }

    fn current_task(&self) -> Option<Arc<dyn KTask>> {
        let task = current_task();
        task.map(|t| t.clone() as Arc<dyn KTask>)
    }
    fn put_task(&self, task: Arc<dyn KTask>) {
        let task = task.downcast_arc::<Task>().map_err(|_| ()).unwrap();
        GLOBAL_TASK_MANAGER.add_task(Arc::new(FifoTask::new(task)));
    }
    fn suspend(&self) {
        do_suspend();
    }

    fn schedule_now(&self, task: Arc<dyn KTask>) {
        schedule_now(task.downcast_arc::<Task>().map_err(|_| ()).unwrap());
    }
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
// pub static SORT_SRC: &[u8] = include_bytes!("../../../tests/testbin-second-stage/sort.src");
