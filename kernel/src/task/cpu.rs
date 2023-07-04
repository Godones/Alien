use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::ops::{Index, IndexMut};

use bitflags::bitflags;
use lazy_static::lazy_static;
use spin::Once;

use kernel_sync::Mutex;
use syscall_table::syscall_func;

use crate::arch;
use crate::config::CPU_NUM;
use crate::fs::vfs;
use crate::sbi::shutdown;
use crate::task::context::Context;
use crate::task::schedule::schedule;
use crate::task::task::{Task, TaskState};
use crate::task::INIT_PROCESS;
use crate::trap::TrapFrame;

#[derive(Debug, Clone)]
pub struct CPU {
    pub task: Option<Arc<Task>>,
    pub context: Context,
}

pub struct CpuManager<const CPUS: usize> {
    cpus: Vec<CPU>,
}

impl<const CPUS: usize> CpuManager<CPUS> {
    pub fn new() -> Self {
        Self {
            cpus: vec![CPU::empty(); CPUS],
        }
    }
}

impl<const CPUS: usize> Index<usize> for CpuManager<CPUS> {
    type Output = CPU;

    fn index(&self, index: usize) -> &Self::Output {
        &self.cpus[index]
    }
}

impl<const CPUS: usize> IndexMut<usize> for CpuManager<CPUS> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.cpus[index]
    }
}

impl CPU {
    const fn empty() -> Self {
        Self {
            task: None,
            context: Context::empty(),
        }
    }
    pub fn take_process(&mut self) -> Option<Arc<Task>> {
        self.task.take()
    }
    pub fn get_context_raw_ptr(&self) -> *const Context {
        &self.context as *const Context
    }
    pub fn get_context_mut_raw_ptr(&mut self) -> *mut Context {
        &mut self.context as *mut Context
    }
}

/// save info for each cpu
static mut CPU_MANAGER: Once<CpuManager<CPU_NUM>> = Once::new();

/// the global process pool
type ProcessPool = VecDeque<Arc<Task>>;
lazy_static! {
    pub static ref PROCESS_MANAGER: Mutex<ProcessPool> = Mutex::new(ProcessPool::new());
}

pub fn init_per_cpu() {
    unsafe {
        CPU_MANAGER.call_once(|| CpuManager::new());
    }
    println!("{} cpus in total", CPU_NUM);
}

/// get the current cpu info
pub fn current_cpu() -> &'static mut CPU {
    let hart_id = arch::hart_id();
    unsafe {
        let cpu_manager = CPU_MANAGER.get_mut().unwrap();
        cpu_manager.index_mut(hart_id)
    }
    // unsafe { &mut CPU_MANAGER[hart_id] }
}

/// get the current_process
pub fn current_task() -> Option<&'static Arc<Task>> {
    let cpu = current_cpu();
    cpu.task.as_ref()
}

/// get the current process's token (root ppn)
pub fn current_user_token() -> usize {
    let task = current_task().unwrap();
    task.token()
}

/// get the current process's trap frame
pub fn current_trap_frame() -> &'static mut TrapFrame {
    let task = current_task().unwrap();
    task.trap_frame()
}

#[syscall_func(93)]
pub fn do_exit(exit_code: i32) -> isize {
    let c_process = current_task().unwrap();
    let exit_code = (exit_code & 0xff) << 8;
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
    c_process.update_state(TaskState::Zombie);
    c_process.update_exit_code(exit_code);
    c_process.recycle();
    schedule();
    0
}

#[syscall_func(94)]
pub fn exit_group(exit_code: i32) -> isize {
    do_exit(exit_code)
}

#[syscall_func(124)]
pub fn do_suspend() -> isize {
    let process = current_task().unwrap();
    process.update_state(TaskState::Ready);
    schedule();
    0
}

#[syscall_func(172)]
pub fn get_pid() -> isize {
    let process = current_task().unwrap();
    process.get_pid()
}

#[syscall_func(173)]
pub fn get_ppid() -> isize {
    let process = current_task().unwrap();
    let parent = process.access_inner().parent.clone();
    if parent.is_none() {
        return 0;
    } else {
        parent.unwrap().upgrade().unwrap().get_pid()
    }
}

#[syscall_func(220)]
pub fn clone(flag: usize, stack: usize, ptid: usize, tls: usize, ctid: usize) -> isize {
    // now we ignore ptid, tls, ctid
    // assert!(ptid == 0 && tls == 0 && ctid == 0);
    let clone_flag = CloneFlags::from_bits_truncate(flag as u32);
    // check whether flag include signal
    let sig = flag & 0xff;
    let sig = SignalFlags::from_bits_truncate(sig as u32);
    let process = current_task().unwrap();
    let new_process = process.t_clone(clone_flag, stack, sig, ptid, tls, ctid);
    if new_process.is_none() {
        return -1;
    }
    let new_process = new_process.unwrap();
    // update return value
    let trap_frame = new_process.trap_frame();
    trap_frame.update_res(0);
    let pid = new_process.get_pid();
    let mut process_pool = PROCESS_MANAGER.lock();
    process_pool.push_back(new_process);
    pid
}

#[syscall_func(221)]
pub fn do_exec(path: *const u8, args_ptr: *const usize, env: *const usize) -> isize {
    let process = current_task().unwrap();
    let str = process.transfer_str(path);
    let mut data = Vec::new();
    // get the args and push them into the new process stack
    let mut args = Vec::new();
    let mut start = args_ptr as *mut usize;
    loop {
        let arg = process.transfer_raw_ptr(start);
        if *arg == 0 {
            break;
        }
        args.push(*arg);
        start = unsafe { start.add(1) };
    }
    let mut args = args
        .into_iter()
        .map(|arg| {
            let mut arg = process.transfer_str(arg as *const u8);
            arg.push('\0');
            arg
        })
        .collect::<Vec<String>>();
    let mut elf_name = str.clone();
    elf_name.push('\0');
    args.insert(0, elf_name);
    // get the env and push them into the new process stack
    let mut envs = Vec::new();
    let mut start = env as *mut usize;
    loop {
        let env = process.transfer_raw_ptr(start);
        if *env == 0 {
            break;
        }
        envs.push(*env);
        start = unsafe { start.add(1) };
    }
    let envs = envs
        .into_iter()
        .map(|env| {
            let mut env = process.transfer_str(env as *const u8);
            env.push('\0');
            env
        })
        .collect::<Vec<String>>();

    if vfs::read_all(&str, &mut data) {
        let res = process.exec(&str, data.as_slice(), args, envs);
        if res.is_err() {
            return res.err().unwrap();
        }
        return 0;
    } else {
        -1
    }
}

/// Please care about the exit code,it may be null
#[syscall_func(260)]
pub fn wait_pid(pid: isize, exit_code: *mut i32, options: u32, _rusage: *const u8) -> isize {
    let process = current_task().unwrap().clone();
    loop {
        if process
            .children()
            .iter()
            .find(|child| child.get_pid() == pid || pid == -1)
            .is_none()
        {
            return -1;
        }
        let children = process.children();
        let res = children.iter().enumerate().find(|(_, child)| {
            child.state() == TaskState::Terminated && (child.get_pid() == pid || pid == -1)
        });
        let res = res.map(|(index, _)| index);
        drop(children);
        if let Some(index) = res {
            let child = process.remove_child(index);
            assert_eq!(Arc::strong_count(&child), 1);
            if !exit_code.is_null() {
                let exit_code_ref = process.transfer_raw_ptr(exit_code);
                *exit_code_ref = child.exit_code();
            }
            return child.get_pid();
        } else {
            let wait_options = WaitOptions::from_bits(options).unwrap();
            if wait_options.contains(WaitOptions::WNOHANG) {
                return 0;
            } else {
                do_suspend();
            }
        }
    }
}

#[syscall_func(214)]
pub fn do_brk(addr: usize) -> isize {
    let process = current_task().unwrap();
    let mut inner = process.access_inner();
    let heap_info = inner.heap_info();
    if addr == 0 {
        return heap_info.current as isize;
    }
    if addr < heap_info.start || addr < heap_info.current {
        return -1;
    }
    let res = inner.extend_heap(addr);
    if res.is_err() {
        return -1;
    }
    res.unwrap() as isize
}

#[syscall_func(96)]
pub fn set_tid_address(tidptr: *mut i32) -> isize {
    let task = current_task().unwrap();
    task.set_tid_address(tidptr as usize);
    task.get_tid()
}

bitflags! {
    pub struct WaitOptions:u32 {
        const WNOHANG = 1;
        const WUNTRACED = 2;
        const WCONTINUED = 8;
    }
}

bitflags! {
    pub struct CloneFlags: u32 {
        const CLONE_VM = 0x00000100;
        const CLONE_FS = 0x00000200;
        const CLONE_FILES = 0x00000400;
        const CLONE_SIGHAND = 0x00000800;
        const CLONE_PTRACE = 0x00002000;
        const CLONE_VFORK = 0x00004000;
        const CLONE_PARENT = 0x00008000;
        const CLONE_THREAD = 0x00010000;
        const CLONE_NEWNS = 0x00020000;
        const CLONE_SYSVSEM = 0x00040000;
        const CLONE_SETTLS = 0x00080000;
        const CLONE_PARENT_SETTID = 0x00100000;
        const CLONE_CHILD_CLEARTID = 0x00200000;
        const CLONE_DETACHED = 0x00400000;
        const CLONE_UNTRACED = 0x00800000;
        const CLONE_CHILD_SETTID = 0x01000000;
        const CLONE_NEWCGROUP = 0x02000000;
        const CLONE_NEWUTS = 0x04000000;
        const CLONE_NEWIPC = 0x08000000;
        const CLONE_NEWUSER = 0x10000000;
        const CLONE_NEWPID = 0x20000000;
        const CLONE_NEWNET = 0x40000000;
        const CLONE_IO = 0x80000000;
    }
}

bitflags! {
    pub struct SignalFlags:u32 {
        const SIGHUP = 1;
        const SIGINT = 2;
        const SIGQUIT = 3;
        const SIGILL = 4;
        const SIGTRAP = 5;
        const SIGABRT = 6;
        const SIGBUS = 7;
        const SIGFPE = 8;
        const SIGKILL = 9;
        const SIGUSR1 = 10;
        const SIGSEGV = 11;
        const SIGUSR2 = 12;
        const SIGPIPE = 13;
        const SIGALRM = 14;
        const SIGTERM = 15;
        const SIGSTKFLT = 16;
        const SIGCHLD = 17;
        const SIGCONT = 18;
        const SIGSTOP = 19;
        const SIGTSTP = 20;
        const SIGTTIN = 21;
        const SIGTTOU = 22;
        const SIGURG = 23;
        const SIGXCPU = 24;
        const SIGXFSZ = 25;
        const SIGVTALRM = 26;
        const SIGPROF = 27;
        const SIGWINCH = 28;
        const SIGIO = 29;
        const SIGPWR = 30;
        const SIGSYS = 31;
    }
}
