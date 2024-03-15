use crate::elf::{build_vm_space, VmmPageAllocator};
use crate::kstack::KStack;
use crate::resource::{FdManager, HeapInfo, TidHandle, UserStack};
use crate::vfs_shim::{ShimFile, STDIN, STDOUT};
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::{Arc, Weak};
use alloc::vec;
use config::{
    FRAME_SIZE, MAX_THREAD_NUM, TRAP_CONTEXT_BASE, USER_KERNEL_STACK_SIZE, USER_STACK_SIZE,
};
use context::{TaskContext, TrapFrame};
use core::ops::Range;
use interface::{InodeId, VFS_ROOT_ID};
use ksync::{Mutex, MutexGuard};
use ptable::VmSpace;
use small_index::IndexAllocator;

#[derive(Debug)]
pub struct Task {
    /// 任务的唯一标识
    pub tid: Arc<TidHandle>,
    /// 作为进程时，pid == tid；作为线程时，pid 为其线程组 leader (父进程)的 tid 号。
    pub pid: Arc<TidHandle>,
    /// 当退出时是否向父进程发送信号 SIGCHLD。
    /// 如果创建时带 CLONE_THREAD 选项，则不发送信号，除非它是线程组(即拥有相同pid的所有线程)中最后一个退出的线程；
    /// 否则发送信号
    pub send_sigchld_when_exit: bool,
    /// 内核栈
    pub kernel_stack: KStack,
    /// 更详细的信息
    pub inner: Mutex<TaskInner>,
}

#[derive(Debug)]
pub struct TaskInner {
    /// 任务名，一般为其文件路径加文件名
    pub name: String,
    /// 线程计数器，用于分配同一个线程组中的线程序号
    pub threads: IndexAllocator<MAX_THREAD_NUM>,
    /// 用于记录当前线程组中的线程个数
    pub thread_number: usize,
    /// 地址空间
    pub address_space: Arc<Mutex<VmSpace<VmmPageAllocator>>>,
    /// 线程状态
    pub state: TaskState,
    /// 父亲任务控制块
    pub parent: Option<Weak<Task>>,
    /// 孩子任务控制块的集合
    pub children: BTreeMap<TidHandle, Arc<Task>>,
    /// 文件描述符表
    pub fd_table: Arc<Mutex<FdManager>>,
    /// 任务上下文
    pub context: TaskContext,
    /// 文件系统的信息
    pub fs_info: FsContext,
    /// 返回值
    pub exit_code: i32,
    /// 堆空间
    pub heap: Arc<Mutex<HeapInfo>>,
    /// 子线程初始化时，存放 tid 的地址。当且仅当创建时包含 CLONE_CHILD_SETTID 才非0
    pub set_child_tid: usize,
    /// 子线程初始化时，将这个地址清空；子线程退出时，触发这里的 futex。
    /// 在创建时包含 CLONE_CHILD_SETTID 时才非0，但可以被 sys_set_tid_address 修改
    pub clear_child_tid: usize,
    /// 栈空间的信息
    pub stack: Range<usize>,
}

#[derive(Debug, Clone)]
pub struct FsContext {
    /// current working directory
    pub cwd: InodeId,
    /// root directory
    pub root: InodeId,
}

impl FsContext {
    pub fn new(cwd: InodeId, root: InodeId) -> Self {
        Self { cwd, root }
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub enum TaskState {
    /// 就绪态
    Ready,
    /// 运行态
    Running,
    /// 等待一个事件
    Waiting,
    /// 僵尸态，等待父进程回收资源
    Zombie,
    /// 终止态
    Terminated,
}

impl Task {
    pub fn inner(&self) -> MutexGuard<TaskInner> {
        self.inner.lock()
    }

    pub fn state(&self) -> TaskState {
        let inner = self.inner.lock();
        inner.state
    }

    pub fn update_state(&self, state: TaskState) {
        let mut inner = self.inner.lock();
        inner.state = state;
    }

    pub fn get_file(&self, fd: usize) -> Option<Arc<ShimFile>> {
        let inner = self.inner.lock();
        let file = inner.fd_table.lock().get(fd);
        file
    }

    pub fn token(&self) -> usize {
        let inner = self.inner.lock();
        let paddr = inner.address_space.lock().root_paddr();
        (8usize << 60) | (paddr >> 12)
    }

    pub fn transfer_raw(&self, ptr: usize) -> usize {
        let address_space = self.inner().address_space.clone();
        let guard = address_space.lock();
        guard
            .query(ptr)
            .map(|(phy_addr, _, _)| phy_addr.as_usize())
            .unwrap()
    }

    pub fn get_context_raw_ptr(&self) -> *const TaskContext {
        let inner = self.inner.lock();
        &inner.context as *const TaskContext
    }

    /// 获取任务上下文的可变指针
    pub fn get_context_mut_raw_ptr(&self) -> *mut TaskContext {
        let mut inner = self.inner.lock();
        &mut inner.context as *mut TaskContext
    }

    /// 获取一个虚拟地址 `ptr` 对应的 T 类型数据 的 可变引用
    pub fn transfer_raw_ptr<T>(&self, ptr: *mut T) -> &'static mut T {
        let ptr = ptr as usize;
        let address_space = self.inner().address_space.clone();
        let (phy_addr, _, _) = address_space.lock().query(ptr).unwrap();
        let phy_addr = phy_addr.as_usize();
        let ptr = phy_addr as *mut T;
        unsafe { &mut *ptr }
    }

    pub fn trap_frame(&self) -> &'static mut TrapFrame {
        let inner = self.inner();
        let trap_context_base = TRAP_CONTEXT_BASE - inner.thread_number * FRAME_SIZE;

        let (physical, _, _) = inner.address_space.lock().query(trap_context_base).unwrap();
        TrapFrame::from_raw_ptr(physical.as_usize() as *mut TrapFrame)
    }
}

impl Task {
    pub fn from_elf(name: &str, elf: &[u8]) -> Option<Task> {
        let tid = Arc::new(TidHandle::new()?);
        let pid = tid.clone();
        let mut args = vec![];
        let elf_info = build_vm_space(elf, &mut args, "init");
        if elf_info.is_err() {
            return None;
        }
        let elf_info = elf_info.unwrap();
        let address_space = elf_info.address_space;
        let k_stack = KStack::new(USER_KERNEL_STACK_SIZE / FRAME_SIZE);
        let k_stack_top = k_stack.top();
        let stack_info = elf_info.stack_top - USER_STACK_SIZE..elf_info.stack_top;

        let trap_to_user = libsyscall::trap_to_user();

        let task = Task {
            tid,
            kernel_stack: k_stack,
            pid,
            inner: Mutex::new(TaskInner {
                name: name.to_string(),
                threads: IndexAllocator::new(),
                thread_number: 0,
                address_space: Arc::new(Mutex::new(address_space)),
                state: TaskState::Ready,
                parent: None,
                children: BTreeMap::new(),
                fd_table: {
                    let mut fd_table = FdManager::new();
                    fd_table.insert(STDIN.clone());
                    fd_table.insert(STDOUT.clone());
                    fd_table.insert(STDOUT.clone());
                    Arc::new(Mutex::new(fd_table))
                },
                context: TaskContext::new(trap_to_user, k_stack_top),
                fs_info: FsContext::new(VFS_ROOT_ID, VFS_ROOT_ID),
                exit_code: 0,
                heap: Arc::new(Mutex::new(HeapInfo::new(
                    elf_info.heap_bottom,
                    elf_info.heap_bottom,
                ))),
                set_child_tid: 0,
                clear_child_tid: 0,
                stack: stack_info,
            }),
            send_sigchld_when_exit: false,
        };
        let phy_button = task.transfer_raw(elf_info.stack_top - FRAME_SIZE);
        let mut user_stack = UserStack::new(phy_button + FRAME_SIZE, elf_info.stack_top);
        user_stack.push(0).unwrap();
        let argc_ptr = user_stack.push(0).unwrap();

        let trap_frame = task.trap_frame();

        let kernel_satp = libsyscall::kernel_satp();
        let user_trap_vector = libsyscall::trap_from_user();

        *trap_frame = TrapFrame::init_for_task(
            elf_info.entry,
            argc_ptr,
            kernel_satp,
            task.kernel_stack.top(),
            user_trap_vector,
        );
        trap_frame.regs()[4] = elf_info.tls; // tp --> tls
        Some(task)
    }
}
