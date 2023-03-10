mod context;
mod cpu;
mod process;
pub mod schedule;
mod stack;
mod thread;

use crate::fs::vfs;
use crate::fs::vfs::{TMP_DIR, TMP_MNT};
use crate::task::cpu::PROCESS_MANAGER;
use crate::task::process::{FsContext, Process};
use alloc::sync::Arc;
use alloc::vec::Vec;
pub use cpu::{
    current_process, current_trap_frame, current_user_token, do_exec, do_exit, do_fork, do_suspend,
    get_pid, wait_pid,
};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref INIT_PROCESS: Arc<Process> = {
        let mut data = Vec::new();
        vfs::read_all("/init", &mut data);
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
    process_pool.push_back(INIT_PROCESS.clone());
    println!("init process success");
}
