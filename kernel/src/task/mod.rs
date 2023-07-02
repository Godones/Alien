use alloc::sync::Arc;
use alloc::vec::Vec;

use lazy_static::lazy_static;

pub use cpu::{
    clone, current_cpu, current_task, current_trap_frame, current_user_token, do_brk, do_exec,
    do_exit, do_suspend, get_pid, get_ppid, init_per_cpu, PROCESS_MANAGER, wait_pid,
};
pub use task::{StatisticalData, Task, TaskState};

use crate::fs::vfs;
use crate::fs::vfs::{TMP_DIR, TMP_MNT};
use crate::task::task::FsContext;

mod context;
mod cpu;
mod task;
pub mod schedule;
mod stack;
mod heap;

lazy_static! {
    pub static ref INIT_PROCESS: Arc<Task> = {
        let mut data = Vec::new();
        vfs::read_all("/init", &mut data);
        let task = Task::from_elf("/init", data.as_slice()).unwrap();
        Arc::new(task)
    };
}

/// put init process into process pool
pub fn init_process() {
    let mut task_pool = PROCESS_MANAGER.lock();
    let task = INIT_PROCESS.clone();
    let dir = TMP_DIR.lock().clone();
    let mnt = TMP_MNT.lock().clone();
    task.access_inner().fs_info =
        FsContext::new(dir.clone(), dir.clone(), mnt.clone(), mnt.clone());
    task_pool.push_back(task);
    println!("init process success");
}
