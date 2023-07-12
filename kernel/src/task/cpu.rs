use alloc::collections::VecDeque;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::ops::{Index, IndexMut};

use lazy_static::lazy_static;
use spin::Once;

use kernel_sync::Mutex;
use syscall_define::ipc::FutexOp;
use syscall_define::signal::SignalNumber;
use syscall_define::task::{CloneFlags, WaitOptions};
use syscall_define::{PrLimit, PrLimitRes};
use syscall_table::syscall_func;

use crate::arch;
use crate::config::CPU_NUM;
use crate::fs::vfs;
use crate::ipc::{futex, global_logoff_signals};
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
    pub static ref TASK_MANAGER: Mutex<ProcessPool> = Mutex::new(ProcessPool::new());
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
    let task = current_task().unwrap();
    let exit_code = (exit_code & 0xff) << 8;
    if task.get_pid() == 0 {
        println!("init process exit with code {}", exit_code);
        shutdown();
    }
    {
        let init = INIT_PROCESS.clone();
        task.take_children().into_iter().for_each(|child| {
            child.update_parent(init.clone());
            init.insert_child(child);
        });
    }
    task.update_state(TaskState::Zombie);
    task.update_exit_code(exit_code);
    global_logoff_signals(task.get_tid() as usize);
    // clear_child_tid 的值不为 0，则将这个用户地址处的值写为0
    let addr = task.access_inner().clear_child_tid;
    if addr != 0 {
        // 确认这个地址在用户地址空间中。如果没有也不需要报错，因为线程马上就退出了
        let addr = task.transfer_raw_ptr(addr as *mut i32);
        *addr = 0;
    }
    task.recycle();
    let clear_child_tid = task.futex_wake();
    if clear_child_tid != 0 {
        let phy_addr = task.transfer_raw_ptr(clear_child_tid as *mut usize);
        *phy_addr = 0;
        error!("exit wake futex on {:#x}", clear_child_tid);
        futex(clear_child_tid, FutexOp::FutexWake as u32, 1, 0, 0, 0);
    } else {
        error!("exit clear_child_tid is 0");
    }
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

#[syscall_func(174)]
pub fn getuid() -> isize {
    0
}

/// 获取有效用户 id，即相当于哪个用户的权限。在实现多用户权限前默认为最高权限
#[syscall_func(175)]
pub fn geteuid() -> isize {
    0
}

/// 获取用户组 id。在实现多用户权限前默认为最高权限
#[syscall_func(176)]
pub fn getgid() -> isize {
    0
}

/// 获取有效用户组 id，即相当于哪个用户的权限。在实现多用户权限前默认为最高权限
#[syscall_func(177)]
pub fn getegid() -> isize {
    0
}

#[syscall_func(178)]
pub fn get_tid() -> isize {
    let process = current_task().unwrap();
    process.get_tid()
}

#[syscall_func(220)]
pub fn clone(flag: usize, stack: usize, ptid: usize, tls: usize, ctid: usize) -> isize {
    let clone_flag = CloneFlags::from_bits_truncate(flag as u32);
    // check whether flag include signal
    let sig = flag & 0xff;
    let sig = SignalNumber::from(sig);
    let task = current_task().unwrap();
    let new_task = task.t_clone(clone_flag, stack, sig, ptid, tls, ctid);
    if new_task.is_none() {
        return -1;
    }
    let new_task = new_task.unwrap();
    // update return value
    let trap_frame = new_task.trap_frame();
    trap_frame.update_res(0);
    let tid = new_task.get_tid();
    let mut process_pool = TASK_MANAGER.lock();
    process_pool.push_back(new_task);
    tid
}

#[syscall_func(221)]
pub fn do_exec(path: *const u8, args_ptr: usize, env: usize) -> isize {
    let task = current_task().unwrap();
    let mut path_str = task.transfer_str(path);
    let mut data = Vec::new();
    // get the args and push them into the new process stack
    let (mut args, envs) = parse_user_arg_env(args_ptr, env);

    if path_str.ends_with(".sh") {
        args.insert(0, path_str.clone());
        path_str = "busybox".to_string();
        args.insert(0, "sh\0".to_string());
    }
    warn!("exec path: {}", path_str);
    if vfs::read_all(&path_str, &mut data) {
        let res = task.exec(&path_str, data.as_slice(), args, envs);
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
pub fn wait4(pid: isize, exit_code: *mut i32, options: u32, _rusage: *const u8) -> isize {
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
            assert_eq!(
                Arc::strong_count(&child),
                1,
                "Father is [{}-{}], wait task is [{}-{}]",
                process.get_pid(),
                process.get_tid(),
                child.get_pid(),
                child.get_tid()
            );
            if !exit_code.is_null() {
                let exit_code_ref = process.transfer_raw_ptr(exit_code);
                *exit_code_ref = child.exit_code();
            }
            return child.get_tid();
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
pub fn set_tid_address(tidptr: usize) -> isize {
    let task = current_task().unwrap();
    task.set_tid_address(tidptr);
    task.get_tid()
}

#[syscall_func(261)]
pub fn prlimit64(pid: usize, resource: usize, new_limit: *const u8, old_limit: *mut u8) -> isize {
    assert!(pid == 0 || pid == current_task().unwrap().get_pid() as usize);
    let task = current_task().unwrap();
    let mut inner = task.access_inner();
    if let Ok(resource) = PrLimitRes::try_from(resource) {
        let limit = inner.get_prlimit(resource);
        if !old_limit.is_null() {
            let old_limit = inner.transfer_raw_ptr_mut(old_limit as *mut PrLimit);
            *old_limit = limit
        }
        match resource {
            PrLimitRes::RlimitStack => {}
            PrLimitRes::RlimitNofile => {
                if !new_limit.is_null() {
                    let new_limit = inner.transfer_raw_ptr(new_limit as *const PrLimit);
                    inner.set_prlimit(resource, *new_limit);
                }
            }
            PrLimitRes::RlimitAs => {}
        }
    }
    0
}

fn parse_user_arg_env(args_ptr: usize, env_ptr: usize) -> (Vec<String>, Vec<String>) {
    let task = current_task().unwrap();
    let mut args = Vec::new();
    let mut start = args_ptr as *mut usize;
    loop {
        let arg = task.transfer_raw_ptr(start);
        if *arg == 0 {
            break;
        }
        args.push(*arg);
        start = unsafe { start.add(1) };
    }
    let args = args
        .into_iter()
        .map(|arg| {
            let mut arg = task.transfer_str(arg as *const u8);
            arg.push('\0');
            arg
        })
        .collect::<Vec<String>>();
    let mut envs = Vec::new();
    let mut start = env_ptr as *mut usize;
    loop {
        let env = task.transfer_raw_ptr(start);
        if *env == 0 {
            break;
        }
        envs.push(*env);
        start = unsafe { start.add(1) };
    }
    let envs = envs
        .into_iter()
        .map(|env| {
            let mut env = task.transfer_str(env as *const u8);
            env.push('\0');
            env
        })
        .collect::<Vec<String>>();
    (args, envs)
}
