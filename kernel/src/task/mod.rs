use alloc::sync::Arc;
use alloc::vec::Vec;

use lazy_static::lazy_static;

pub use cpu::{
    clone, current_cpu, current_process, current_trap_frame, current_user_token, do_brk, do_exec,
    do_exit, do_suspend, get_pid, get_ppid, init_per_cpu, PROCESS_MANAGER, wait_pid,
};
pub use process::{Process, ProcessState, StatisticalData};

use crate::fs::vfs;
use crate::fs::vfs::{TMP_DIR, TMP_MNT};
use crate::task::process::FsContext;

mod context;
mod cpu;
mod process;
pub mod schedule;
mod stack;
mod thread;

lazy_static! {
    pub static ref INIT_PROCESS: Arc<Process> = {
        let mut data = Vec::new();
        vfs::read_all("/final/time-test", &mut data);
        let process = Process::from_elf(data.as_slice()).unwrap();
        Arc::new(process)
    };
}

/// put init process into process pool
pub fn init_process() {
    let mut process_pool = PROCESS_MANAGER.lock();
    let process = INIT_PROCESS.clone();
    let dir = TMP_DIR.lock().clone();
    let mnt = TMP_MNT.lock().clone();
    process.access_inner().fs_info =
        FsContext::new(dir.clone(), dir.clone(), mnt.clone(), mnt.clone());
    process_pool.push_back(process);
    println!("init process success");
}
