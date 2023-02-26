use alloc::boxed::Box;
use crate::fs::File;
use alloc::sync::{Arc, Weak};
use gmanager::MinimalManager;
use lazy_static::lazy_static;
use page_table::AddressSpace;

use crate::config::{MAX_FD_NUM, MAX_PROCESS_NUM, MAX_SUB_PROCESS_NUM, MAX_THREAD_NUM};
use crate::memory::build_elf_address_space;
use crate::task::thread::Thread;
use spin::Mutex;
use crate::task::context::Context;
use crate::task::stack::Stack;
use crate::trap::TrapFrame;

type ThreadManager = MinimalManager<Arc<Thread>>;
type SubProcessManager = MinimalManager<Arc<Process>>;
type FdManager = MinimalManager<Arc<dyn File>>;

lazy_static! {
    /// 这里把MinimalManager复用为pid分配器，通常，MinimalManager会将数据插入到最小可用位置并返回位置，
    /// 但pid的分配并不需要实际存储信息，因此可以插入任意的数据，这里为了节省空间，将数据定义为u8
    pub static ref PID_MANAGER:Mutex<MinimalManager<u8>> = Mutex::new(MinimalManager::new(MAX_PROCESS_NUM));
}

pub struct PidHandle(usize);
impl Drop for PidHandle {
    fn drop(&mut self) {
        PID_MANAGER.lock().remove(self.0).unwrap();
    }
}

/// 进程
pub struct Process {
    pub pid: PidHandle,
    pub inner: Mutex<ProcessInner>,
}

pub struct ProcessInner {
    pub address_space: AddressSpace,
    pub state: ProcessState,
    pub father: Option<Weak<Process>>,
    pub children: SubProcessManager,
    pub trap_frame: Box<TrapFrame>,
    pub fd_table: FdManager,
    pub context: Context,
    pub kernel_stack: Stack,
    pub exit_code: i32,
}

pub enum ProcessState {
    Init,
    Running,
    Sleeping,
    Zombie,
}

impl Process {
    pub fn new(elf: &[u8]) -> Option<Process> {
        let pid = PID_MANAGER.lock().insert(0).unwrap();
        // 创建进程地址空间
        let elf_info = build_elf_address_space(elf);
        if elf_info.is_err() {
            return None;
        }
        let elf_info = elf_info.unwrap();
        let process = Process {
            pid: PidHandle(pid),
            inner: Mutex::new(ProcessInner {
                address_space: elf_info.address_space,
                state: ProcessState::Init,
                father: None,
                children: SubProcessManager::new(MAX_SUB_PROCESS_NUM),
                trap_frame: Box::new(TrapFrame::empty()),
                fd_table: FdManager::new(MAX_FD_NUM),
                context: Context::new(0,0),
                kernel_stack: Stack::new(1)?,
                exit_code: 0,
            }),
        };
        // 创建主线程

        Some(process)
    }
}
