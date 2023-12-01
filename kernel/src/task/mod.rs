//! Alien 中有关进程管理的相关数据结构
//!
//! [`context`] 子模块定义了 Alien 中线程上下文的相关结构.
//! [`cpu`] 子模块中指明了 Alien 中有关进程的系统调用 和 多核的相关支持。
//! [`heap`] 子模块定义了 Alien 记录进程堆空间的相关信息的结构。
//! [`schedule`] 子模块指明了 Alien 中有关 CPU 调度的相关机制
//! [`stack`] 子模块定义了 Alien 中有关内核栈的相关结构。
//! [`task`] 子模块定义了 Alien 中有关进程控制块的定义。
use alloc::sync::Arc;
use alloc::vec::Vec;
use smpscheduler::FifoTask;

use spin::Lazy;

pub use cpu::*;
pub use task::{StatisticalData, Task, TaskState};

use crate::fs::{read_all, SYSTEM_ROOT_FS};
pub use crate::task::task::FsContext;

mod context;
mod cpu;
mod heap;
pub mod schedule;
mod stack;
mod task;

/// 初始进程（0号进程）
pub static INIT_PROCESS: Lazy<Arc<Task>> = Lazy::new(|| {
    let mut data = Vec::new();
    read_all("/bin/init", &mut data);
    // let data = INIT;
    assert!(data.len()>0);
    let task = Task::from_elf("/bin/init", data.as_slice()).unwrap();
    Arc::new(task)
});

/// 将初始进程加入进程池中进行调度
pub fn init_process() {
    let task = INIT_PROCESS.clone();
    let cwd = SYSTEM_ROOT_FS.get().unwrap().clone();
    let root = cwd.clone();
    task.access_inner().fs_info = FsContext::new(root, cwd);
    GLOBAL_TASK_MANAGER.add_task(Arc::new(FifoTask::new(task)));
    println!("Init process success");
}
// online test has no sort.src
// pub static SORT_SRC: &[u8] = include_bytes!("../../../sdcard/sort.src");
