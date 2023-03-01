mod context;
mod cpu;
mod process;
pub mod schedule;
mod stack;
mod thread;

use crate::fs;
use crate::task::cpu::PROCESS_MANAGER;
use crate::task::process::Process;
use alloc::sync::Arc;
pub use cpu::{
    current_process, current_trap_frame, current_user_token, do_exec, do_exit, do_fork, do_suspend,
    get_pid, wait_pid,
};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref INIT_PROCESS: Arc<Process> = {
        let init = fs::open_file("init").unwrap();
        let file_size = init.size();
        let data = init.read(0, file_size).expect("read init file failed");
        let process = Process::from_elf(data.as_slice()).unwrap();
        Arc::new(process)
    };
}

/// put init process into process pool
pub fn init_process() {
    let mut process_pool = PROCESS_MANAGER.lock();
    process_pool.push_back(INIT_PROCESS.clone());
}
