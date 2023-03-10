use crate::arch;
use crate::config::CPU_NUM;
use crate::fs::vfs;
use crate::sbi::shutdown;
use crate::sync::IntrLock;
use crate::task::context::Context;
use crate::task::process::{Process, ProcessState};
use crate::task::schedule::schedule;
use crate::task::INIT_PROCESS;
use crate::trap::TrapFrame;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::Mutex;
#[derive(Debug)]
pub struct CPU {
    pub process: Option<Arc<Process>>,
    pub context: Context,
    pub intr_lock: IntrLock,
}

impl CPU {
    const fn empty() -> Self {
        Self {
            process: None,
            context: Context::empty(),
            intr_lock: IntrLock::new(),
        }
    }
    pub fn take_process(&mut self) -> Option<Arc<Process>> {
        self.process.take()
    }
    pub fn get_context_raw_ptr(&self) -> *const Context {
        &self.context as *const Context
    }
}
/// save info for each cpu
static mut CPU_MANAGER: [CPU; CPU_NUM] = [CPU::empty(); CPU_NUM];

/// the global process pool
type ProcessPool = VecDeque<Arc<Process>>;
lazy_static! {
    pub static ref PROCESS_MANAGER: Mutex<ProcessPool> = Mutex::new(ProcessPool::new());
}

/// get the current cpu info
pub fn current_cpu() -> &'static mut CPU {
    let hart_id = arch::hart_id();
    unsafe { &mut CPU_MANAGER[hart_id] }
}

/// get the current_process
pub fn current_process() -> Option<&'static Arc<Process>> {
    let cpu = current_cpu();
    cpu.process.as_ref()
}

/// get the current process's token (root ppn)
pub fn current_user_token() -> usize {
    let process = current_process().unwrap();
    process.token()
}

/// get the current process's trap frame
pub fn current_trap_frame() -> &'static mut TrapFrame {
    let process = current_process().unwrap();
    process.trap_frame()
}

pub fn do_exit(exit_code: i32) -> isize {
    let c_process = current_process().unwrap();
    if c_process.get_pid() == 0 {
        println!("init process exit with code {}", exit_code);
        shutdown();
    }
    {
        let init = INIT_PROCESS.clone();
        c_process.children().iter().for_each(|child| {
            child.update_parent(init.clone());
            init.insert_child(child.clone());
        });
    }
    c_process.update_state(ProcessState::Zombie);
    c_process.update_exit_code(exit_code);
    c_process.recycle();
    schedule();
    0
}

pub fn do_suspend() -> isize {
    let process = current_process().unwrap();
    process.update_state(ProcessState::Sleeping);
    schedule();
    0
}

pub fn get_pid() -> isize {
    let process = current_process().unwrap();
    process.get_pid()
}

pub fn do_fork() -> isize {
    let process = current_process().unwrap();
    let new_process = process.fork();
    if new_process.is_none() {
        return -1;
    }
    let new_process = new_process.unwrap();
    let mut process_pool = PROCESS_MANAGER.lock();
    // update return value
    let trap_frame = new_process.trap_frame();
    trap_frame.update_res(0);
    let pid = new_process.get_pid();
    process_pool.push_back(new_process);
    pid
}

pub fn do_exec(path: *const u8) -> isize {
    let process = current_process().unwrap();
    let mut str = process.transfer_str(path);
    let mut data = Vec::new();
    if !str.starts_with("/") {
        str.insert(0, '/');
    }
    if vfs::read_all(&str, &mut data) {
        let res = process.exec(data.as_slice());
        if res.is_err() {
            return res.err().unwrap() as isize;
        }
    }
    0
}

pub fn wait_pid(pid: isize, exit_code: *mut i32) -> isize {
    let process = current_process().unwrap();
    if !process
        .children()
        .iter()
        .any(|child| child.get_pid() == pid || pid == -1)
    {
        return -1;
    }
    let children = process.children();
    let res = children.iter().enumerate().find(|(_, child)| {
        child.state() == ProcessState::Zombie && (child.get_pid() == pid || pid == -1)
    });
    match res {
        Some((index, _)) => {
            drop(children);
            let child = process.remove_child(index);
            assert_eq!(Arc::strong_count(&child), 1);
            let exit_code_ref = process.transfer_raw_ptr(exit_code);
            *exit_code_ref = child.exit_code();
            child.get_pid() as isize
        }
        _ => -2,
    }
}
