use alloc::{collections::BTreeMap, string::ToString, sync::Arc, vec::Vec};

use bit_field::BitField;
use config::{CPU_NUM, FRAME_SIZE, MAX_FD_NUM, MAX_THREAD_NUM, USER_KERNEL_STACK_SIZE};
use constants::{
    ipc::RobustList,
    signal::{SignalHandlers, SignalReceivers},
    AlienError, AlienResult,
};
use gmanager::MinimalManager;
use ksync::Mutex;
use mem::kernel_space;
use smpscheduler::FifoTask;
use vfs::kfile::File;

use crate::{
    fs::stdio::{STDIN, STDOUT},
    mm::map::MMapInfo,
    task::{
        context::Context,
        resource::{HeapInfo, TidHandle},
        stack::Stack,
        task::{TaskInner, TaskTimer},
        FsContext, StatisticalData, Task, TaskState, GLOBAL_TASK_MANAGER,
    },
};

type FdManager = MinimalManager<Arc<dyn File>>;

pub fn ktread_create(func: fn(), name: &str) -> AlienResult<()> {
    let tid = TidHandle::new().ok_or(AlienError::ENOSPC)?;
    let pid = tid.0;
    let k_stack = Stack::new(USER_KERNEL_STACK_SIZE / FRAME_SIZE).ok_or(AlienError::ENOMEM)?;
    let kspace = kernel_space();
    let cwd = vfs::system_root_fs();
    let k_stack_top = k_stack.top();
    let func_ptr = func as usize;
    let task = Task {
        tid,
        kernel_stack: k_stack,
        pid,
        inner: Mutex::new(TaskInner {
            name: name.to_string(),
            threads: MinimalManager::new(MAX_THREAD_NUM),
            thread_number: 0,
            address_space: kspace,
            state: TaskState::Ready,
            parent: None,
            children: Vec::new(),
            fd_table: {
                let mut fd_table = FdManager::new(MAX_FD_NUM);
                fd_table.insert(STDIN.clone()).unwrap();
                fd_table.insert(STDOUT.clone()).unwrap();
                fd_table.insert(STDOUT.clone()).unwrap();
                Arc::new(Mutex::new(fd_table))
            },
            context: Context::new(func_ptr, k_stack_top),
            fs_info: FsContext::new(cwd.clone(), cwd),
            statistical_data: StatisticalData::new(),
            timer: TaskTimer::default(),
            exit_code: 0,
            heap: Arc::new(Mutex::new(HeapInfo::new(0, 0))),
            mmap: MMapInfo::new(),
            signal_handlers: Arc::new(Mutex::new(SignalHandlers::new())),
            signal_receivers: Arc::new(Mutex::new(SignalReceivers::new())),
            set_child_tid: 0,
            clear_child_tid: 0,
            trap_cx_before_signal: None,
            signal_set_siginfo: false,
            robust: RobustList::default(),
            shm: BTreeMap::new(),
            cpu_affinity: {
                let mut affinity = 0;
                affinity.set_bits(0..CPU_NUM, 1 << CPU_NUM - 1);
                affinity
            },
            unmask: 0o022,
            // user mode stack info
            stack: 0..0,
            need_wait: 0,
        }),
        send_sigchld_when_exit: false,
    };
    let task = Arc::new(task);
    let task = Arc::new(FifoTask::new(task));
    GLOBAL_TASK_MANAGER.add_task(task);
    Ok(())
}
