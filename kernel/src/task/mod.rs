use alloc::sync::Arc;

use lazy_static::lazy_static;

pub use cpu::*;
pub use task::{StatisticalData, Task, TaskState};

use crate::fs::vfs::{TMP_DIR, TMP_MNT};
use crate::task::task::FsContext;

mod context;
mod cpu;
mod heap;
pub mod schedule;
mod stack;
mod task;

lazy_static! {
    pub static ref INIT_PROCESS: Arc<Task> = {
        // let mut data = Vec::new();
        // vfs::read_all("/bin/init", &mut data);
        let data = INIT;
        let task = Task::from_elf("/bin/init", data).unwrap();
        Arc::new(task)
    };
}
/// put init process into process pool
pub fn init_process() {
    let mut task_pool = TASK_MANAGER.lock();
    let task = INIT_PROCESS.clone();
    let dir = TMP_DIR.lock().clone();
    let mnt = TMP_MNT.lock().clone();
    task.access_inner().fs_info =
        FsContext::new(dir.clone(), dir.clone(), mnt.clone(), mnt.clone());
    task_pool.push_back(task);
    println!("init process success");
}

static INIT: &[u8] = include_bytes!("../../../target/riscv64gc-unknown-none-elf/release/init");

static LIBC_BENCH2: &[u8] = include_bytes!("../../../sdcard/libc-bench2");
// online test has no sort.src
pub static SORT_SRC: &[u8] = include_bytes!("../../../sdcard/sort.src");
