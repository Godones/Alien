//! Alien 中任务控制块的相关定义。
//!
//! Alien 中对于进程\线程的相关概念的设计与 Linux 类似，进程和线程共用一个控制块结构。
//! 使用 `clone` 创建新的进程(线程)时，会根据 flag 指明父子进程之间资源共享的程度。
//! tid 是标识不同任务的唯一标识。
use crate::fs::stdio::{STDIN, STDOUT};
use crate::ipc::{global_register_signals, ShmInfo};
use crate::mm::loader::{
    build_cow_address_space, build_elf_address_space, build_thread_address_space, UserStack,
};
use crate::mm::map::{MMapInfo, MMapRegion, ProtFlags};
use crate::task::context::Context;
use crate::task::heap::HeapInfo;
use crate::task::stack::Stack;
use crate::trap::{trap_common_read_file, trap_return, user_trap_vector, TrapFrame};
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use alloc::{format, vec};
use bit_field::BitField;
use config::*;
use constants::aux::*;
use constants::io::MapFlags;
use constants::ipc::RobustList;
use constants::signal::{SignalHandlers, SignalNumber, SignalReceivers, SignalUserContext};
use constants::sys::TimeVal;
use constants::task::CloneFlags;
use constants::time::TimerType;
use constants::{AlienError, AlienResult};
use constants::{LinuxErrno, PrLimit, PrLimitRes};
use core::fmt::{Debug, Formatter};
use core::ops::Range;
use gmanager::MinimalManager;
use ksync::{Mutex, MutexGuard};
use mem::{kernel_satp, VmmPageAllocator, FRAME_REF_MANAGER};
use page_table::addr::{align_down_4k, align_up_4k, VirtAddr};
use page_table::pte::MappingFlags;
use page_table::table::Sv39PageTable;
use spin::Lazy;
use timer::{read_timer, ITimerVal, TimeNow, ToClock};
use vfs::kfile::File;
use vfscore::dentry::VfsDentry;

type FdManager = MinimalManager<Arc<dyn File>>;

/// 这里把MinimalManager复用为tid分配器，通常，MinimalManager会将数据插入到最小可用位置并返回位置，
/// 但tid的分配并不需要实际存储信息，因此可以插入任意的数据，这里为了节省空间，将数据定义为u8
pub static TID_MANAGER: Lazy<Mutex<MinimalManager<u8>>> =
    Lazy::new(|| Mutex::new(MinimalManager::new(MAX_THREAD_NUM)));

/// 用于存储线程的tid
#[derive(Debug)]
pub struct TidHandle(pub usize);

impl TidHandle {
    /// 获取一个新的线程 tid (来自于 `TID_MANAGER` 分配)
    pub fn new() -> Option<Self> {
        let tid = TID_MANAGER.lock().insert(0);
        if tid.is_err() {
            return None;
        }
        Some(Self(tid.unwrap()))
    }
}

impl Drop for TidHandle {
    fn drop(&mut self) {
        TID_MANAGER.lock().remove(self.0).unwrap();
    }
}

#[derive(Debug)]
pub struct Task {
    /// 任务的唯一标识
    pub tid: TidHandle,
    /// 作为进程时，pid == tid；作为线程时，pid 为其线程组 leader (父进程)的 tid 号。
    pub pid: usize,
    /// 当退出时是否向父进程发送信号 SIGCHLD。
    /// 如果创建时带 CLONE_THREAD 选项，则不发送信号，除非它是线程组(即拥有相同pid的所有线程)中最后一个退出的线程；
    /// 否则发送信号
    pub send_sigchld_when_exit: bool,
    /// 内核栈
    pub kernel_stack: Stack,
    /// 更详细的信息
    pub inner: Mutex<TaskInner>,
}

#[derive(Debug)]
pub struct TaskInner {
    /// 任务名，一般为其文件路径加文件名
    pub name: String,
    /// 线程计数器，用于分配同一个线程组中的线程序号
    pub threads: MinimalManager<()>,
    /// 用于记录当前线程组中的线程个数
    pub thread_number: usize,
    /// 地址空间
    pub address_space: Arc<Mutex<Sv39PageTable<VmmPageAllocator>>>,
    /// 线程状态
    pub state: TaskState,
    /// 父亲任务控制块
    pub parent: Option<Weak<Task>>,
    /// 孩子任务控制块的集合
    pub children: Vec<Arc<Task>>,
    /// 文件描述符表
    pub fd_table: Arc<Mutex<FdManager>>,
    /// 任务上下文
    pub context: Context,
    /// 文件系统的信息
    pub fs_info: FsContext,
    /// 有关任务执行情况的统计信息
    pub statistical_data: StatisticalData,
    /// 任务计时器
    pub timer: TaskTimer,
    /// 返回值
    pub exit_code: i32,
    /// 堆空间
    pub heap: Arc<Mutex<HeapInfo>>,
    /// 地址空间中的映射信息
    pub mmap: MMapInfo,
    /// 信号量对应的一组处理函数。
    /// 因为发送信号是通过 pid/tid 查找的，因此放在 inner 中一起调用时更容易导致死锁
    pub signal_handlers: Arc<Mutex<SignalHandlers>>,
    /// 接收信号的结构。每个线程中一定是独特的，而上面的 handler 可能是共享的
    pub signal_receivers: Arc<Mutex<SignalReceivers>>,
    /// 子线程初始化时，存放 tid 的地址。当且仅当创建时包含 CLONE_CHILD_SETTID 才非0
    pub set_child_tid: usize,
    /// 子线程初始化时，将这个地址清空；子线程退出时，触发这里的 futex。
    /// 在创建时包含 CLONE_CHILD_SETTID 时才非0，但可以被 sys_set_tid_address 修改
    pub clear_child_tid: usize,
    /// 处理信号时，保存的之前的用户线程的上下文信息
    pub trap_cx_before_signal: Option<TrapFrame>,
    /// 保存信息时，处理函数是否设置了 SIGINFO 选项
    /// 如果设置了，说明信号触发前的上下文信息通过 ucontext 传递给了用户，
    /// 此时用户可能修改其中的 pc 信息(如musl-libc 的 pthread_cancel 函数)。
    /// 在这种情况下，需要手动在 sigreturn 时更新已保存的上下文信息
    pub signal_set_siginfo: bool,
    /// robust 锁的列表
    pub robust: RobustList,
    /// 共享内存
    pub shm: BTreeMap<usize, ShmInfo>,
    /// cpu 亲和力，用于 cpu 调度时 倾向于将该任务调度给 哪个 CPU
    pub cpu_affinity: usize,
    /// 进程创建文件时，文件权限的默认掩码
    pub unmask: usize,
    /// 栈空间的信息
    pub stack: Range<usize>,
    /// 是否需要等待
    pub need_wait: u8,
}

#[derive(Debug, Copy, Clone)]
pub struct TaskTimer {
    /// 计时器类型
    pub timer_type: TimerType,
    /// 设置下一次触发计时器的区间
    ///
    /// 当 timer_remained 归零时，如果 timer_interval 非零，则将其重置为 timer_interval 的值；
    /// 否则，则这个计时器不再触发
    pub timer_interval: TimeVal,
    /// 当前计时器还剩下多少时间。
    ///
    /// 根据 timer_type 的规则不断减少，当归零时触发信号
    pub timer_remained: usize,
    /// 上一次计时的开始时间
    pub start: usize,
    /// 该计时器是否已经超时
    pub expired: bool,
}

impl TaskTimer {
    /// 清除当前的计数器信息，将 timer_remained 置为 0
    pub fn clear(&mut self) {
        self.timer_type = TimerType::NONE;
        self.timer_interval = TimeVal::new();
        self.timer_remained = 0;
        self.expired = false;
    }
}

impl Default for TaskTimer {
    /// 默认的任务计数器
    fn default() -> Self {
        Self {
            timer_type: TimerType::NONE,
            timer_interval: TimeVal::new(),
            timer_remained: 0,
            start: 0,
            expired: false,
        }
    }
}

/// statistics of a process
#[derive(Debug, Clone)]
pub struct StatisticalData {
    /// The number of times the process was scheduled in user mode. --ticks
    pub tms_utime: usize,
    /// The number of times the process was scheduled in kernel mode. --ticks
    pub tms_stime: usize,
    /// The last time the process was scheduled in user mode. --ticks
    pub last_utime: usize,
    /// The last time the process was scheduled in kernel mode. --ticks
    pub last_stime: usize,

    pub tms_cutime: usize,
    pub tms_cstime: usize,
}

impl StatisticalData {
    /// 用于创建一个新的 `StatisticalData` 结构
    pub fn new() -> Self {
        let now = read_timer();
        StatisticalData {
            tms_utime: 0,
            tms_stime: 0,
            last_utime: now,
            last_stime: now,
            tms_cutime: 0,
            tms_cstime: 0,
        }
    }
    /// 清除当前 `StatisticalData` 结构中储存的数据，并将 `last_utime` 和 `last_stime` 的值置为 当前的时间
    pub fn clear(&mut self) {
        let now = read_timer();
        self.tms_utime = 0;
        self.tms_stime = 0;
        self.last_utime = now;
        self.last_stime = now;
        self.tms_cutime = 0;
        self.tms_cstime = 0;
    }
}

#[derive(Clone)]
pub struct FsContext {
    /// 当前工作目录
    pub cwd: Arc<dyn VfsDentry>,
    /// 根目录
    pub root: Arc<dyn VfsDentry>,
}

impl Debug for FsContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let cwd = self.cwd.path();
        let root = self.root.path();
        f.debug_struct("FsContext")
            .field("cwd", &cwd)
            .field("root", &root)
            .finish()
    }
}

impl FsContext {
    /// 创建一个新的 `FsContext` 结构
    pub fn new(root: Arc<dyn VfsDentry>, cwd: Arc<dyn VfsDentry>) -> Self {
        FsContext { cwd, root }
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub enum TaskState {
    /// 就绪态
    Ready,
    /// 运行态
    Running,
    /// 等待一段时间
    // Sleeping,
    /// 等待一个事件
    Waiting,
    /// 僵尸态，等待父进程回收资源
    Zombie,
    /// 终止态
    Terminated,
}

impl Task {
    /// 终止进程，回收内核栈等资源，同时将该任务的状态修改为 `Terminated`
    pub fn terminate(self: Arc<Self>) {
        // recycle kernel stack
        self.kernel_stack.release();
        if self.access_inner().thread_number != 0 {
            let parent = self.inner.lock().parent.clone();
            if let Some(parent) = parent {
                let parent = parent.upgrade();
                if let Some(parent) = parent {
                    parent.remove_child_by_tid(self.get_tid());
                    assert_eq!(Arc::strong_count(&self), 1);
                }
            }
        } else {
        }
        self.access_inner().state = TaskState::Terminated;
    }

    /// 获取进程的 pid 号
    #[inline]
    pub fn get_pid(&self) -> isize {
        self.pid as isize
    }

    /// 获取进程的 tid 号
    #[inline]
    pub fn get_tid(&self) -> isize {
        self.tid.0 as isize
    }

    /// 设置 `clear_child_tid` 字段的 值
    pub fn set_tid_address(&self, tidptr: usize) {
        let mut inner = self.inner.lock();
        inner.clear_child_tid = tidptr;
    }

    /// 获取文件的名称
    pub fn get_name(&self) -> String {
        let inner = self.inner.lock();
        inner.name.clone()
    }

    /// 尝试获取 TaskInner
    pub fn access_inner(&self) -> MutexGuard<TaskInner> {
        self.inner.lock()
    }

    /// 获取进程页表的root ppn
    pub fn token(&self) -> usize {
        let inner = self.inner.lock();
        let paddr = inner.address_space.lock().root_paddr();
        (8usize << 60) | (paddr.as_usize() >> 12)
    }

    /// 获取 trap 帧的可变引用
    pub fn trap_frame(&self) -> &'static mut TrapFrame {
        self.inner.lock().trap_frame()
    }

    /// 获取 trap 帧的指针
    pub fn trap_frame_ptr(&self) -> *mut TrapFrame {
        self.inner.lock().trap_frame_ptr()
    }

    /// 将进程的状态修改为 state
    pub fn update_state(&self, state: TaskState) {
        let mut inner = self.inner.lock();
        inner.state = state;
    }

    /// 返回进程的状态
    pub fn state(&self) -> TaskState {
        let inner = self.inner.lock();
        inner.state
    }

    /// 更新进程的返回码
    pub fn update_exit_code(&self, code: i32) {
        let mut inner = self.inner.lock();
        inner.exit_code = code;
    }

    /// 获取任务上下文的指针
    pub fn get_context_raw_ptr(&self) -> *const Context {
        let inner = self.inner.lock();
        &inner.context as *const Context
    }

    /// 获取任务上下文的可变指针
    pub fn get_context_mut_raw_ptr(&self) -> *mut Context {
        let mut inner = self.inner.lock();
        &mut inner.context as *mut Context
    }

    /// 获取该进程的子进程的控制块列表
    pub fn children(&self) -> Vec<Arc<Task>> {
        let inner = self.inner.lock();
        inner.children.clone()
    }

    /// 查找进程号为pid的子进程是否已经退出。当pid为-1时，任意子进程退出都将返回子进程的pid号。
    pub fn check_child(&self, pid: isize) -> Option<usize> {
        let res = self
            .inner
            .lock()
            .children
            .iter()
            .enumerate()
            .find(|(_, child)| {
                child.state() == TaskState::Terminated && (child.get_pid() == pid || pid == -1)
            })
            .map(|(index, _)| index);
        res
    }

    /// 取走当前进程的子进程控制块列表的所有权
    pub fn take_children(&self) -> Vec<Arc<Task>> {
        let children = self.children();
        self.access_inner().children = Vec::new();
        children
    }

    /// 将任务号为 `tid` 的进程 从子进程列表中移除
    pub fn remove_child_by_tid(&self, tid: isize) -> Option<Arc<Task>> {
        let mut inner = self.inner.lock();
        let index = inner
            .children
            .iter()
            .position(|child| child.get_tid() == tid);
        if let Some(index) = index {
            Some(inner.children.remove(index))
        } else {
            None
        }
    }

    /// 将子进程列表中第 `index` 个进程 从子进程列表中移除
    pub fn remove_child(&self, index: usize) -> Arc<Task> {
        let mut inner = self.inner.lock();
        assert!(index < inner.children.len());
        inner.children.remove(index)
    }

    /// 将进程的父进程更新为 `parent`
    pub fn update_parent(&self, parent: Arc<Task>) {
        let mut inner = self.inner.lock();
        inner.parent = Some(Arc::downgrade(&parent));
    }

    /// 向子进程列表中插入一个新的子进程
    pub fn insert_child(&self, child: Arc<Task>) {
        let mut inner = self.inner.lock();
        inner.children.push(child);
    }

    /// 获取进程当前的返回码
    pub fn exit_code(&self) -> i32 {
        let inner = self.inner.lock();
        inner.exit_code
    }

    /// 用于获取文件描述符id号为 fd 的 文件描述符
    pub fn get_file(&self, fd: usize) -> Option<Arc<dyn File>> {
        let inner = self.inner.lock();
        let file = inner.fd_table.lock().get(fd);
        return if file.is_err() { None } else { file.unwrap() };
    }

    /// 在进程的文件描述符表中加入 file 文件
    pub fn add_file(&self, file: Arc<dyn File>) -> Result<usize, isize> {
        self.access_inner()
            .fd_table
            .lock()
            .insert(file)
            .map_err(|x| x as isize)
    }

    /// 指定文件描述符表中的一个id，在该处加入一个 file 文件
    pub fn add_file_with_fd(&self, file: Arc<dyn File>, fd: usize) -> Result<(), ()> {
        let inner = self.access_inner();
        let mut fd_table = inner.fd_table.lock();
        fd_table.insert_with_index(fd, file).map_err(|_| {})
    }

    /// 指明文件描述符表中的一个id，删除并返回该处的 file 文件
    pub fn remove_file(&self, fd: usize) -> Result<Arc<dyn File>, ()> {
        let inner = self.inner.lock();
        let file = inner.fd_table.lock().get(fd);
        if file.is_err() {
            return Err(());
        }
        let file = file.unwrap();
        if file.is_none() {
            return Err(());
        }
        let file = file.unwrap();
        inner.fd_table.lock().remove(fd).map_err(|_| {})?;
        Ok(file)
    }

    /// 获取一个虚拟地址 `ptr` 的实际物理地址
    pub fn transfer_raw(&self, ptr: usize) -> usize {
        self.access_inner().transfer_raw(ptr)
    }

    /// 获取一个虚拟地址 `ptr` 对应的 T 类型数据 的 可变引用
    pub fn transfer_raw_ptr<T>(&self, ptr: *mut T) -> &'static mut T {
        self.access_inner().transfer_raw_ptr_mut(ptr)
    }

    /// 通过用户地址空间中一个字符串的首指针 `ptr`，获取一个字符串。
    pub fn transfer_str(&self, ptr: *const u8) -> String {
        // we need check the ptr and len before transfer indeed
        let mut start = ptr as usize;
        let end = start + 128; //todo! string len is unknown
        let address_space = self.access_inner().address_space.clone();
        while start < end {
            let (_phy, flag, _) = address_space
                .lock()
                .query(VirtAddr::from(start))
                .expect(format!("transfer_buffer: {:x} failed", start).as_str());
            if !flag.contains(MappingFlags::V) {
                error!("transfer_str flag: {:?}, addr:{:#x}", flag, start);
                let res = self
                    .access_inner()
                    .invalid_page_solver(align_down_4k(start))
                    .unwrap();
                if res.is_some() {
                    let (file, buf, offset) = res.unwrap();
                    if file.is_some() {
                        trap_common_read_file(file.unwrap(), buf, offset);
                    }
                }
            }
            start += FRAME_SIZE;
        }
        self.access_inner().transfer_str(ptr)
    }

    /// 通过用户地址空间中一个缓冲区的指针 `ptr` 和 缓冲区的长度 `len`，得到一组对用户地址空间中缓冲区的可变引用，每一组引用的长度为 4K
    pub fn transfer_buffer<T: Debug>(&self, ptr: *const T, len: usize) -> Vec<&'static mut [T]> {
        // we need check the ptr and len before transfer indeed
        let mut start = ptr as usize;
        let end = start + len;
        let address_space = self.access_inner().address_space.clone();
        start = align_down_4k(start);
        while start < end {
            let (_phy, flag, _) = address_space
                .lock()
                .query(VirtAddr::from(start))
                .expect(format!("transfer_buffer: {:x} failed", start).as_str());
            if !flag.contains(MappingFlags::V) {
                error!("transfer_buffer flag: {:?}, addr:{:#x}", flag, start);
                let res = self
                    .access_inner()
                    .invalid_page_solver(align_down_4k(start))
                    .unwrap();
                if res.is_some() {
                    let (file, buf, offset) = res.unwrap();
                    if file.is_some() {
                        trap_common_read_file(file.unwrap(), buf, offset);
                    }
                }
            }
            start += FRAME_SIZE;
        }
        self.access_inner().transfer_buffer(ptr, len)
    }
}

impl TaskInner {
    /// 获取进程的文件系统信息
    pub fn cwd(&self) -> FsContext {
        self.fs_info.clone()
    }

    /// 获取进程的计时器
    pub fn get_timer(&self) -> TaskTimer {
        self.timer.clone()
    }

    /// 获取当前进程对于资源的限制
    pub fn get_prlimit(&self, resource: PrLimitRes) -> PrLimit {
        match resource {
            PrLimitRes::RlimitStack => PrLimit::new(USER_STACK_SIZE as u64, USER_STACK_SIZE as u64),
            PrLimitRes::RlimitNofile => {
                let max_fd = self.fd_table.lock().max();
                PrLimit::new(max_fd as u64, max_fd as u64)
            }
            PrLimitRes::RlimitAs => PrLimit::new(u64::MAX, u64::MAX),
        }
    }

    /// 设置当前进程对于资源的限制
    pub fn set_prlimit(&mut self, resource: PrLimitRes, value: PrLimit) {
        match resource {
            PrLimitRes::RlimitStack => {}
            PrLimitRes::RlimitNofile => {
                let new_max_fd = value.rlim_cur;
                self.fd_table.lock().set_max(new_max_fd as usize);
            }
            PrLimitRes::RlimitAs => {}
        }
    }

    /// 返回 trap 上下文的一个可变指针
    pub fn trap_frame_ptr(&self) -> *mut TrapFrame {
        let trap_context_base = if self.thread_number != 0 {
            let base = TRAP_CONTEXT_BASE - self.thread_number * FRAME_SIZE;
            base
        } else {
            TRAP_CONTEXT_BASE
        };
        trap_context_base as *mut TrapFrame
    }

    /// 返回 trap 上下文的一个可变引用
    pub fn trap_frame(&self) -> &'static mut TrapFrame {
        let trap_context_base = if self.thread_number != 0 {
            let base = TRAP_CONTEXT_BASE - self.thread_number * FRAME_SIZE;
            base
        } else {
            TRAP_CONTEXT_BASE
        };
        let (physical, _, _) = self
            .address_space
            .lock()
            .query(VirtAddr::from(trap_context_base))
            .unwrap();
        TrapFrame::from_raw_ptr(physical.as_usize() as *mut TrapFrame)
    }

    /// 在信号处理需要执行用户态信号处理函数时，保存原 trap 上下文。
    pub fn save_trap_frame(&mut self) -> bool {
        let trap_frame = self.trap_frame();
        if self.trap_cx_before_signal.is_some() {
            return false;
        }
        self.trap_cx_before_signal = Some(*trap_frame);
        self.signal_set_siginfo = false;
        true
    }

    /// 待用户态信号处理函数执行完毕后，需要重新加载原 trap 上下文。
    pub fn load_trap_frame(&mut self) -> isize {
        if let Some(old_trap_frame) = self.trap_cx_before_signal.take() {
            let trap_frame = self.trap_frame();
            // 这里假定是 sigreturn 触发的，即用户的信号处理函数 return 了(cancel_handler)
            // 也就是说信号触发时的 sp 就是现在的 sp
            let sp = trap_frame.regs()[2];
            // 获取可能被修改的 pc
            let phy_sp = self.transfer_raw(sp);

            let pc = unsafe { (*(phy_sp as *const SignalUserContext)).get_pc() };
            *trap_frame = old_trap_frame;
            if self.signal_set_siginfo {
                // 更新用户修改的 pc
                trap_frame.set_sepc(pc);
                warn!("sig return sp = {:x} pc = {:x}", sp, pc);
            }
            trap_frame.regs()[10] as isize // old arg0
        } else {
            -1
        }
    }

    /// 获取一个虚拟地址 `ptr` 的实际物理地址
    pub fn transfer_raw(&mut self, ptr: usize) -> usize {
        let (phy, flag, _) = self
            .address_space
            .lock()
            .query(VirtAddr::from(ptr))
            .unwrap();
        if !flag.contains(MappingFlags::V) {
            error!("[transfer_raw] invalid page {:?}, ptr:{:#x}", flag, ptr);
            self.invalid_page_solver(ptr).unwrap();
            let (phy, flag, _) = self
                .address_space
                .lock()
                .query(VirtAddr::from(ptr))
                .unwrap();
            assert!(flag.contains(MappingFlags::V));
            return phy.as_usize();
        }
        phy.as_usize()
    }

    /// 获取 虚拟地址空间中的以 `ptr` 为起始地址，以 '\0' 结尾的字符串
    pub fn transfer_str(&self, ptr: *const u8) -> String {
        let mut res = String::new();
        let physical = self
            .address_space
            .lock()
            .query(VirtAddr::from(ptr as usize));
        if physical.is_err() {
            return res;
        }
        let (physical, _, _) = physical.unwrap();
        let mut physical = physical.as_usize();
        loop {
            let c = unsafe { &*(physical as *const u8) };
            if *c == 0 {
                break;
            }
            res.push(*c as char);
            physical += 1;
        }
        res
    }

    /// 从物理地址的 `src` 处取一个长度为 `len` 类型为 T 的缓冲区 赋到 用户虚拟地址空间下的 `dst` 处
    pub fn copy_to_user_buffer<T: 'static + Copy>(
        &mut self,
        src: *const T,
        dst: *mut T,
        len: usize,
    ) {
        let size = core::mem::size_of::<T>() * len;
        if VirtAddr::from(dst as usize).align_down_4k()
            == VirtAddr::from(dst as usize + size - 1).align_down_4k()
        {
            // the src and dst are in same page
            let dst = self.transfer_raw(dst as usize);
            unsafe {
                core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, size);
            }
        } else {
            let bufs = self.transfer_buffer(dst as *const u8, size);
            let src = unsafe { core::slice::from_raw_parts(src as *const u8, size) };
            let mut start = 0;
            let src_len = src.len();
            for buffer in bufs {
                let len = if start + buffer.len() > src_len {
                    src_len - start
                } else {
                    buffer.len()
                };
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        src.as_ptr().add(start),
                        buffer.as_mut_ptr(),
                        len,
                    );
                }
                start += len;
            }
        }
    }

    /// 从用户虚拟地址空间的 `src` 处取一个长度为 `len` 类型为 T 的缓冲区 赋到 物理地址下的 `dst` 处
    pub fn copy_from_user_buffer<T: 'static + Copy>(
        &mut self,
        src: *const T,
        dst: *mut T,
        len: usize,
    ) {
        let size = core::mem::size_of::<T>() * len;
        if VirtAddr::from(src as usize).align_down_4k()
            == VirtAddr::from(src as usize + size - 1).align_down_4k()
        {
            // the src and dst are in same page
            let src = self.transfer_raw(src as usize);
            unsafe {
                core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, size);
            }
        } else {
            let mut bufs = self.transfer_buffer(src as *const u8, size);
            let dst = unsafe { core::slice::from_raw_parts_mut(dst as *mut u8, size) };
            let mut start = 0;
            let dst_len = dst.len();
            for buffer in bufs.iter_mut() {
                let len = if start + buffer.len() > dst_len {
                    dst_len - start
                } else {
                    buffer.len()
                };
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        buffer.as_ptr(),
                        dst.as_mut_ptr().add(start),
                        len,
                    );
                }
                start += len;
            }
        }
    }

    /// 从物理空间下的 `src` 处取一个 T 类型的数据 赋给 虚拟地址空间下的 `dst` 处
    pub fn copy_to_user<T: 'static + Copy>(&mut self, src: *const T, dst: *mut T) {
        // self.copy_to_user_buffer(src, dst, 1);
        let size = core::mem::size_of::<T>();
        if VirtAddr::from(dst as usize).align_down_4k()
            == VirtAddr::from(dst as usize + size - 1).align_down_4k()
        {
            // the src and dst are in same page
            let dst = self.transfer_raw(dst as usize);
            unsafe {
                core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, size);
            }
        } else {
            let bufs = self.transfer_buffer(dst as *const u8, size);
            let src = unsafe { core::slice::from_raw_parts(src as *const u8, size) };
            let mut start = 0;
            let src_len = src.len();
            for buffer in bufs {
                let len = if start + buffer.len() > src_len {
                    src_len - start
                } else {
                    buffer.len()
                };
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        src.as_ptr().add(start),
                        buffer.as_mut_ptr(),
                        len,
                    );
                }
                start += len;
            }
        }
    }

    /// 从用户虚拟地址空间的 `src` 处取一个 T 类型的数据 赋给 物理地址下的 `dst` 处
    pub fn copy_from_user<T: 'static + Copy>(&mut self, src: *const T, dst: *mut T) {
        // self.copy_from_user_buffer(src, dst, 1);
        let size = core::mem::size_of::<T>();
        if VirtAddr::from(src as usize).align_down_4k()
            == VirtAddr::from(src as usize + size - 1).align_down_4k()
        {
            // the src and dst are in same page
            let src = self.transfer_raw(src as usize);
            unsafe {
                core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, size);
            }
        } else {
            let mut bufs = self.transfer_buffer(src as *const u8, size);
            let dst = unsafe { core::slice::from_raw_parts_mut(dst as *mut u8, size) };
            let mut start = 0;
            let dst_len = dst.len();
            for buffer in bufs.iter_mut() {
                let len = if start + buffer.len() > dst_len {
                    dst_len - start
                } else {
                    buffer.len()
                };
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        buffer.as_ptr(),
                        dst.as_mut_ptr().add(start),
                        len,
                    );
                }
                start += len;
            }
        }
    }

    /// 将在进程的虚拟空间中的一段缓冲区的首地址 `ptr` 和 长度 `len` 转换为 实地址下的一组页
    pub fn transfer_buffer<T: Debug>(
        &mut self,
        ptr: *const T,
        len: usize,
    ) -> Vec<&'static mut [T]> {
        let mut start = ptr as usize;
        let end = start + len;
        let mut v = Vec::new();
        while start < end {
            let (start_phy, flag, _) = self
                .address_space
                .lock()
                .query(VirtAddr::from(start))
                .expect(format!("transfer_buffer: {:x} failed", start).as_str());
            if !flag.contains(MappingFlags::V) {
                panic!("transfer_buffer: {:x} not mapped", start);
            }
            // start_phy向上取整到FRAME_SIZE
            let bound = (start & !(FRAME_SIZE - 1)) + FRAME_SIZE;
            let len = if bound > end {
                end - start
            } else {
                bound - start
            };
            unsafe {
                let buf = core::slice::from_raw_parts_mut(start_phy.as_usize() as *mut T, len);
                v.push(buf);
            }
            start = bound;
        }
        v
    }

    /// 将一个在进程的虚拟空间中的虚地址 转换为一个实地址的可变引用
    pub fn transfer_raw_ptr_mut<T>(&self, ptr: *mut T) -> &'static mut T {
        let (physical, flag, _) = self
            .address_space
            .lock()
            .query(VirtAddr::from(ptr as usize))
            .expect(format!("{} transfer_raw_ptr: {:#x} failed", self.name, ptr as usize).as_str());
        assert!(flag.contains(MappingFlags::V));
        unsafe { &mut *(physical.as_usize() as *mut T) }
    }

    /// 将一个在进程的虚拟空间中的虚地址 转换为一个实地址的不可变引用
    pub fn transfer_raw_ptr<T>(&self, ptr: *const T) -> &'static T {
        let (physical, flag, _) = self
            .address_space
            .lock()
            .query(VirtAddr::from(ptr as usize))
            .expect(format!("{} transfer_raw_ptr: {:#x} failed", self.name, ptr as usize).as_str());
        assert!(flag.contains(MappingFlags::V));
        unsafe { &*(physical.as_usize() as *const T) }
    }

    /// 当进程回到用户态时，需要更新进程在内核态下的运行时间
    /// WARNING: If the cause of the process returning to the kernel is a timer interrupt,
    /// We should not call this function.
    pub fn update_kernel_mode_time(&mut self) {
        let now = read_timer(); // current cpu clocks
        let time = now - self.statistical_data.last_stime;
        self.update_timer();
        self.statistical_data.tms_stime += time;
        self.statistical_data.last_utime = now;
    }

    /// 当进程进入内核态时，需要更新进程在用户态下的运行时间
    pub fn update_user_mode_time(&mut self) {
        let now = read_timer(); // current cpu clocks
        let time = now - self.statistical_data.last_utime;
        self.update_timer();
        self.statistical_data.tms_utime += time;
        self.statistical_data.last_stime = now;
    }

    /// 设置计时器
    pub fn set_timer(&mut self, itimer: ITimerVal, timer_type: TimerType) {
        self.timer.timer_remained = itimer.it_value.to_clock();
        self.timer.timer_interval = itimer.it_interval;
        self.timer.timer_type = timer_type;
        self.timer.start = TimeVal::now().to_clock();
    }

    /// 更新计时器
    ///
    /// 如果没有计时器则直接返回；如果有计时器但时辰未到也直接返回；
    /// 如果有计时器且计时器到时间了，根据是否为one-shot计时器，确定重置计时器或者置`timer`的`timer_remained`为0。
    pub fn update_timer(&mut self) {
        let now = read_timer();
        let delta = now - self.timer.start;
        if self.timer.timer_remained == 0 {
            // 等于0说明没有计时器，或者 one-shot 计时器已结束
            return;
        }
        if self.timer.timer_remained > delta {
            // 时辰未到
            return;
        }
        // 到此说明计时器已经到时间了，更新计时器
        // 如果是 one-shot 计时器，则 timer_interval_us == 0，这样赋值也恰好是符合语义的
        self.timer.timer_remained = if self.timer.timer_interval == TimeVal::new() {
            0
        } else {
            self.timer.start = now;
            self.timer.timer_interval.to_clock()
        };
        self.timer.expired = true;
    }

    /// 在调用 `update_user_mode_time` 和 `update_kernel_mode_time` 后，我们需要检查一下计时器是否已经超时
    pub fn check_timer_expired(&mut self) -> Option<TimerType> {
        if self.timer.expired {
            self.timer.expired = false;
            Some(self.timer.timer_type)
        } else {
            None
        }
    }

    /// 返回进程的统计信息
    pub fn statistical_data(&self) -> &StatisticalData {
        &self.statistical_data
    }

    /// 返回堆信息
    pub fn heap_info(&self) -> HeapInfo {
        self.heap.lock().clone()
    }

    #[allow(unused)]
    /// (待实现)缩减堆空间
    pub fn shrink_heap(_addr: usize) -> Result<usize, AlienError> {
        todo!()
    }

    /// 拓展堆空间
    pub fn extend_heap(&mut self, addr: usize) -> Result<usize, AlienError> {
        let mut heap = self.heap.lock();
        heap.current = addr;
        if addr < heap.end {
            return Ok(heap.current);
        }
        let addition = addr - heap.end;
        // increase heap size
        let end = heap.end;
        // align addition to PAGE_SIZE
        let addition = (addition + FRAME_SIZE - 1) & !(FRAME_SIZE - 1);
        trace!("extend heap: {:#x} -- {:#x}", end, addition);
        self.address_space
            .lock()
            .map_region_no_target(
                VirtAddr::from(end),
                addition,
                "RWUAD".into(), // no V flag
                false,
                true,
            )
            .unwrap();
        let new_end = end + addition;
        heap.end = new_end;
        Ok(heap.current)
    }

    /// 在虚拟空间中创建内存映射。
    /// + `start`: 所要创建的映射区的起始地址。当该值为0时，内核将自动为其分配一段内存空间创建内存映射。该值在函数运行过程中将被调整为与4K对齐。
    /// + `len`: 指明所要创建的映射区的长度。该值在函数运行过程中将被调整为与4K对齐。
    /// + `prot`: 指明创建内存映射区的初始保护位。具体可见[`ProtFlags`]。
    /// + `flags`: 指明mmap操作的相关设置。具体可见[`MapFlags`]。
    /// + `fd`: 指明要创建内存映射的文件的文件描述符。
    /// + `offset`: 将从文件中偏移量为`offset`处开始映射。该值需要和4K对齐。
    ///
    /// 函数成功执行后将返回所创建的内存映射区的首地址；否则返回错误类型。
    pub fn add_mmap(
        &mut self,
        start: usize,
        len: usize,
        prot: ProtFlags,
        flags: MapFlags,
        fd: usize,
        offset: usize,
    ) -> AlienResult<usize> {
        // start == 0 表明需要OS为其找一段内存，而 MAP_FIXED 表明必须 mmap 在固定位置。两者是冲突的
        if start == 0 && flags.contains(MapFlags::MAP_FIXED) {
            return Err(LinuxErrno::EINVAL);
        }

        // if the map in heap, now we ignore it
        if self.heap.lock().contains(start) && self.heap.lock().contains(start + len) {
            return Ok(start);
        }

        // not map to file
        let fd = if flags.contains(MapFlags::MAP_ANONYMOUS) {
            None
        } else {
            let file = self
                .fd_table
                .lock()
                .get(fd)
                .map_err(|_| LinuxErrno::EBADF)?
                .ok_or(LinuxErrno::EBADF)?; // EBADF
            Some(file)
        };
        // todo!
        // for dynamic link, the linker will map the elf file to the same address
        // we must satisfy this requirement
        let mut start = align_down_4k(start);
        let v_range = if prot.contains(ProtFlags::PROT_EXEC) {
            let len = align_up_4k(len);
            if start > self.heap.lock().start {
                // the mmap region is in heap
                return Err(LinuxErrno::EINVAL);
            }
            if let Some(_region) = self.mmap.get_region(start) {
                return Err(LinuxErrno::EINVAL);
            }
            if start == 0 {
                start = 0x1000;
            }
            start..start + len
        } else if flags.contains(MapFlags::MAP_FIXED) {
            let len = align_up_4k(len);
            if start > self.heap.lock().start {
                error!("mmap fixed address conflict with heap");
                return Err(LinuxErrno::EINVAL);
            }
            // check if the region is already mapped
            if let Some(region) = self.mmap.get_region(start) {
                // split the region
                let (left, mut right) = region.split(start);
                // delete the old region
                self.mmap.remove_region(region.start);
                // add the left region
                self.mmap.add_region(left);
                if start + len < right.start + right.map_len {
                    // slice the right region
                    trace!(
                        "again slice the right region:{:#x?}, len:{:#x}",
                        right.start,
                        right.len
                    );
                    let (mut left, right) = right.split(start + len);
                    // add the right region
                    self.mmap.add_region(right);
                    // update prot and flags
                    left.set_prot(prot);
                    left.set_flags(flags);
                    left.offset = offset;
                    left.fd = fd;
                    self.mmap.add_region(left);
                } else {
                    trace!(
                        "directly add the right region:{:#x?}, len:{:#x}",
                        right.start,
                        right.len
                    );
                    // update prot and flags
                    right.set_prot(prot);
                    right.set_flags(flags);
                    right.offset = offset;
                    right.fd = fd;
                    self.mmap.add_region(right);
                }
                return Ok(start);
            }
            start..start + len
        } else {
            let v_range = self.mmap.alloc(len);
            v_range
        };

        let region = MMapRegion::new(
            v_range.start,
            len,
            v_range.end - v_range.start,
            prot,
            flags,
            fd,
            offset,
        );
        // warn!("add mmap region:{:#x?}",region);
        self.mmap.add_region(region);
        let start = v_range.start;
        let mut map_flags: MappingFlags = prot.into(); // no V  flag
        map_flags |= "AD".into();
        self.address_space
            .lock()
            .map_region_no_target(
                VirtAddr::from(start),
                v_range.end - start,
                map_flags,
                false,
                true,
            )
            .unwrap();
        Ok(start)
    }

    /// 用于在进程的虚拟内存空间中消除一段内存映射。传入的`start`需要是某段内存映射的首地址，`len`需要是该段内存映射的长度。
    pub fn unmap(&mut self, start: usize, len: usize) -> Result<(), isize> {
        // check whether the start is in mmap
        let x = self.mmap.get_region(start);
        if x.is_none() {
            return Err(LinuxErrno::EINVAL.into());
        }
        // now we need make sure the start is equal to the start of the region, and the len is equal to the len of the region
        let region = x.unwrap();
        if region.start != start || len != region.len {
            return Err(LinuxErrno::EINVAL.into());
        }
        self.address_space
            .lock()
            .unmap_region(VirtAddr::from(start), region.map_len)
            .unwrap();
        self.mmap.remove_region(start);
        Ok(())
    }

    /// 设置内存映射的保护位，函数会检查传入的`start`和`len`所指示的内存映射区是否已经处于被映射状态，如果是，则将对应内存映射区的保护位与`prot`做或运算。
    pub fn map_protect(&mut self, start: usize, len: usize, prot: ProtFlags) -> AlienResult<()> {
        // check whether the start is in mmap
        let x = self.mmap.get_region_mut(start);
        if x.is_none() {
            let res = self.address_space.lock().query(VirtAddr::from(start));
            return if res.is_err() {
                Err(LinuxErrno::EINVAL)
            } else {
                Ok(())
            };
        }
        // now we need make sure the start is equal to the start of the region, and the len is equal to the len of the region
        let region = x.unwrap();
        if start + len > region.start + region.len {
            error!("start+len > region.start + region.len");
            return Err(LinuxErrno::EINVAL);
        }
        region.prot |= prot;
        Ok(())
    }

    /// 用于处理装入页异常
    pub fn do_load_page_fault(
        &mut self,
        addr: usize,
    ) -> AlienResult<Option<(Option<Arc<dyn File>>, &'static mut [u8], u64)>> {
        // check whether the addr is in mmap
        let addr = align_down_4k(addr);
        let (_phy, flags, page_size) = self
            .address_space
            .lock()
            .query(VirtAddr::from(addr))
            .expect(format!("addr:{:#x}", addr).as_str());
        trace!(
            "do load page fault:{:#x}, flags:{:?}, page_size:{:?}",
            addr,
            flags,
            page_size
        );
        if !flags.contains(MappingFlags::V) {
            return self.invalid_page_solver(addr);
        }
        assert!(!flags.contains(MappingFlags::RSD));

        let region = self.mmap.get_region(addr).ok_or(AlienError::EINVAL)?;
        // now we need make sure the start is equal to the start of the region, and the len is equal to the len of the region
        // update page table
        let mut map_flags: MappingFlags = region.prot.into();
        map_flags |= "V".into();

        let mut address_space = self.address_space.lock();

        let (_, flags, _) = address_space.query(VirtAddr::from(addr)).unwrap();
        assert!(!flags.contains(MappingFlags::V));
        address_space
            .validate(VirtAddr::from(addr), map_flags)
            .unwrap();
        let (phy, _, size) = address_space.query(VirtAddr::from(addr)).unwrap();
        let buf =
            unsafe { core::slice::from_raw_parts_mut(phy.as_usize() as *mut u8, size.into()) };
        let file = &region.fd;

        let read_offset = region.offset + (addr - region.start);
        Ok(Some((file.clone(), buf, read_offset as u64)))
    }

    /// 用于处理无效页错误
    fn invalid_page_solver(
        &mut self,
        addr: usize,
    ) -> AlienResult<Option<(Option<Arc<dyn File>>, &'static mut [u8], u64)>> {
        trace!("invalid page fault at {:#x}", addr);
        let is_mmap = self.mmap.get_region(addr);
        let is_heap = self.heap.lock().contains(addr);

        let is_stack = self.stack.contains(&addr);

        if is_mmap.is_none() && !is_heap && !is_stack {
            warn!("invalid page fault at {:#x}", addr);
            return Err(AlienError::EINVAL);
        }
        if is_heap {
            trace!("invalid page fault in heap");
            let map_flags = "RWUVAD".into();
            self.address_space
                .lock()
                .validate(VirtAddr::from(addr), map_flags)
                .unwrap();
        } else if is_mmap.is_some() {
            let region = is_mmap.unwrap();
            // assert_eq!(addr % FRAME_SIZE, 0);
            // update page table
            let mut map_flags: MappingFlags = region.prot.into();
            map_flags |= "VAD".into();
            warn!(
                "invalid page fault at {:#x}, flag is :{:?}",
                addr, map_flags
            );
            self.address_space
                .lock()
                .validate(VirtAddr::from(addr).align_down_4k(), map_flags)
                .unwrap();
            let (phy, flag, size) = self
                .address_space
                .lock()
                .query(VirtAddr::from(addr))
                .unwrap();
            assert!(flag.contains(MappingFlags::V));
            let buf =
                unsafe { core::slice::from_raw_parts_mut(phy.as_usize() as *mut u8, size.into()) };
            let file = &region.fd;
            let read_offset = region.offset + (addr - region.start);
            return Ok(Some((file.clone(), buf, read_offset as u64)));
        } else {
            warn!("invalid page fault in stack, addr: {:#x}", addr);
            let map_flags = "RWUVAD".into();
            self.address_space
                .lock()
                .validate(VirtAddr::from(addr), map_flags)
                .unwrap();
        }
        Ok(None)
    }

    /// 用于处理指令页异常
    pub fn do_instruction_page_fault(
        &mut self,
        addr: usize,
    ) -> AlienResult<Option<(Option<Arc<dyn File>>, &'static mut [u8], u64)>> {
        let addr = align_down_4k(addr);
        let (_phy, flags, page_size) = self
            .address_space
            .lock()
            .query(VirtAddr::from(addr))
            .map_err(|_| AlienError::EINVAL)?;
        //
        trace!(
            "do store page fault:{:#x}, flags:{:?}, page_size:{:?}",
            addr,
            flags,
            page_size
        );
        if !flags.contains(MappingFlags::V) {
            return self.invalid_page_solver(addr);
        }
        panic!("instruction page fault");
    }

    /// 用于处理数据页异常
    pub fn do_store_page_fault(
        &mut self,
        o_addr: usize,
    ) -> AlienResult<Option<(Option<Arc<dyn File>>, &'static mut [u8], u64)>> {
        let addr = align_down_4k(o_addr);
        let (phy, flags, page_size) = self
            .address_space
            .lock()
            .query(VirtAddr::from(addr))
            .map_err(|_x| {
                if self.need_wait < 5 {
                    self.need_wait += 1;
                    AlienError::EAGAIN
                } else {
                    error!("do_store_page_fault panic :{:#x}", o_addr);
                    AlienError::ETMP
                }
            })?;
        // .expect(format!("addr:{:#x}", addr).as_str());
        trace!(
            "do store page fault:{:#x}, flags:{:?}, page_size:{:?}",
            addr,
            flags,
            page_size
        );
        if !flags.contains(MappingFlags::V) {
            return self.invalid_page_solver(addr);
        }
        // if !flags.contains(MappingFlags::RSD) {
        //     return Ok(None);
        // }
        assert!(
            flags.contains(MappingFlags::RSD),
            "addr:{:#x} flags:{:?}",
            o_addr,
            flags
        );
        // decrease the reference count
        let mut flags = flags | "W".into();
        flags -= MappingFlags::RSD;
        let new_phy = self
            .address_space
            .lock()
            .modify_pte_flags(VirtAddr::from(addr), flags, true)
            .unwrap();
        assert!(new_phy.is_some());
        // copy data
        let src_ptr = phy.as_usize() as *const u8;
        let dst_ptr = new_phy.unwrap().as_usize() as *mut u8;
        unsafe {
            core::ptr::copy(src_ptr, dst_ptr, usize::from(page_size));
        }
        let mut frame_ref_manager = FRAME_REF_MANAGER.lock();
        for i in 0..usize::from(page_size) / FRAME_SIZE {
            let t_phy = phy + i * FRAME_SIZE;
            frame_ref_manager.dec_ref(t_phy.as_usize() >> FRAME_BITS);
        }
        Ok(None)
    }
}

impl Task {
    /// 对进程的资源进行预回收，将会回收 trap 帧、子进程控制块列表、文件描述符表等资源。
    pub fn pre_recycle(&self) {
        // recycle trap page
        let trap_frame_ptr = self.trap_frame_ptr() as usize;
        self.access_inner()
            .address_space
            .lock()
            .unmap_region(VirtAddr::from(trap_frame_ptr), FRAME_SIZE)
            .unwrap();
        let mut inner = self.inner.lock();
        // delete child process
        inner.children.clear();
        let thread_number = inner.thread_number;
        if thread_number == 0 {
            let _ = inner.fd_table.lock().clear();
            drop(inner);
        }
    }

    /// 获取进程的 `clear_child_tid` 字段
    pub fn futex_wake(&self) -> usize {
        self.access_inner().clear_child_tid
    }

    /// 从 elf 文件中创建一个新的进程控制块，只会调用一次(即读取 init 进程的相关信息)
    pub fn from_elf(name: &str, elf: &[u8]) -> Option<Task> {
        let tid = TidHandle::new()?;
        let pid = tid.0;
        // 创建进程地址空间
        let mut args = vec![];
        let elf_info = build_elf_address_space(elf, &mut args, "/bin/init");
        if elf_info.is_err() {
            return None;
        }
        let elf_info = elf_info.unwrap();
        let address_space = elf_info.address_space;
        let k_stack = Stack::new(USER_KERNEL_STACK_SIZE / FRAME_SIZE)?;
        let k_stack_top = k_stack.top();
        let stack_info = elf_info.stack_top - USER_STACK_SIZE..elf_info.stack_top;
        let cwd = vfs::system_root_fs();

        let process = Task {
            tid,
            kernel_stack: k_stack,
            pid,
            inner: Mutex::new(TaskInner {
                name: name.to_string(),
                threads: MinimalManager::new(MAX_THREAD_NUM),
                thread_number: 0,
                address_space: Arc::new(Mutex::new(address_space)),
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
                context: Context::new(trap_return as usize, k_stack_top),
                fs_info: FsContext::new(cwd.clone(), cwd),
                statistical_data: StatisticalData::new(),
                timer: TaskTimer::default(),
                exit_code: 0,
                heap: Arc::new(Mutex::new(HeapInfo::new(
                    elf_info.heap_bottom,
                    elf_info.heap_bottom,
                ))),
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
                stack: stack_info,
                need_wait: 0,
            }),
            send_sigchld_when_exit: false,
        };
        let phy_button = process.transfer_raw(elf_info.stack_top - FRAME_SIZE);
        let mut user_stack = UserStack::new(phy_button + FRAME_SIZE, elf_info.stack_top);
        user_stack.push(0).unwrap();
        let argc_ptr = user_stack.push(0).unwrap();

        let trap_frame = process.trap_frame();
        *trap_frame = TrapFrame::init_for_task(
            elf_info.entry,
            argc_ptr,
            kernel_satp(),
            process.kernel_stack.top(),
            user_trap_vector as usize,
        );
        trap_frame.regs()[4] = elf_info.tls; // tp --> tls
        let res = Some(process);
        res
    }

    /// 产生一个新的子进程。
    ///
    /// `flag`用于控制父子进程之间资源的共享程度，有关flag值及其相关含义设置可见[`CloneFlags`]。
    /// `stack`用于控制子进程的用户栈。由于clone产生的子进程有可能和父进程共享内存，所以它不能使用父进程的栈。
    /// `sig`用于控制子进程退出时传递给父进程的相关信号。目前Alien中的设计为当其值为`SIGCHLD`时，在子进程退出时会向父程序发送`SIGCHLD`信号。会其它有关值的设置可见[`SignalNumber`]。
    /// `ptid`是一个在父进程地址空间中的地址，用于在创建子进程成功后向该位置写入子进程的tid号。在flag包含`CLONE_PARENT_SETTID`时才会发挥效果。
    /// `tls`用于为子进程创建新的TLS(thread-local storage)值，在flag包含`CLONE_SETTLS`时才会实际产生效果。
    /// `ctid`用于给子进程中的[`set_child_tid`]和[`clear_child_tid`]赋值(分别在flag中包含`CLONE_CHILD_SETTID`和`CLONE_CHILD_CLEARTID`时产生效果)。
    ///
    /// 成功创建子进程后父进程会返回子进程的TCB。
    ///
    /// Note: 当传入的ptid未在父进程地址空间中被分配时，会引发panic。
    pub fn t_clone(
        self: &Arc<Self>,
        flag: CloneFlags,
        stack: usize,
        sig: SignalNumber,
        ptid: usize,
        tls: usize,
        ctid: usize,
    ) -> Option<Arc<Task>> {
        warn!(
            "clone: flag:{:?}, sig:{:?}, stack:{:#x}, ptid:{:#x}, tls:{:#x}, ctid:{:#x}",
            flag, sig, stack, ptid, tls, ctid
        );
        let tid = TidHandle::new()?;
        let mut inner = self.inner.lock();
        let address_space = if flag.contains(CloneFlags::CLONE_VM) {
            // to create thread
            inner.address_space.clone()
        } else {
            // to create process
            let address_space =
                build_cow_address_space(&mut inner.address_space.lock(), inner.shm.clone());
            Arc::new(Mutex::new(address_space))
        };

        let fd_table = if flag.contains(CloneFlags::CLONE_FILES) {
            inner.fd_table.clone()
        } else {
            Arc::new(Mutex::new(inner.fd_table.lock().clone()))
        };

        let signal_handlers = if flag.contains(CloneFlags::CLONE_SIGHAND) {
            inner.signal_handlers.clone()
        } else {
            Arc::new(Mutex::new(inner.signal_handlers.lock().clone()))
        };

        let parent = if flag.contains(CloneFlags::CLONE_PARENT) {
            inner.parent.clone()
        } else {
            Some(Arc::downgrade(self))
        };

        let k_stack = Stack::new(USER_KERNEL_STACK_SIZE / FRAME_SIZE)?;
        let k_stack_top = k_stack.top();
        let pid = if flag.contains(CloneFlags::CLONE_THREAD) {
            self.pid
        } else {
            tid.0
        };
        let signal_receivers = Arc::new(Mutex::new(SignalReceivers::new()));
        // 注册线程-信号对应关系
        global_register_signals(tid.0, signal_receivers.clone());
        // map the thread trap_context if clone_vm
        let (trap_context, thread_num) = if flag.contains(CloneFlags::CLONE_VM) {
            let thread_num = inner.threads.insert(()).unwrap() + 1;
            warn!("thread_num: {}", thread_num);
            // calculate the address for thread context
            let trap_context = build_thread_address_space(&mut address_space.lock(), thread_num);
            (trap_context, thread_num)
        } else {
            let (physical, _, _) = address_space
                .lock()
                .query(VirtAddr::from(TRAP_CONTEXT_BASE))
                .unwrap();
            let trap_frame = TrapFrame::from_raw_ptr(physical.as_usize() as *mut TrapFrame);
            (trap_frame, 0)
        };

        let heap = if flag.contains(CloneFlags::CLONE_VM) {
            inner.heap.clone()
        } else {
            Arc::new(Mutex::new(inner.heap.lock().clone()))
        };

        // 设置内核栈地址
        trap_context.update_kernel_sp(k_stack_top);

        // 检查是否需要设置 tls
        if flag.contains(CloneFlags::CLONE_SETTLS) {
            trap_context.update_tp(tls);
        }

        // 检查是否在父任务地址中写入 tid
        if flag.contains(CloneFlags::CLONE_PARENT_SETTID) {
            // 有可能这个地址是 lazy alloc 的，需要先检查
            let res = inner.address_space.lock().query(VirtAddr::from(ptid));
            if res.is_ok() {
                let (physical, _, _) = res.unwrap();
                unsafe {
                    *(physical.as_usize() as *mut i32) = tid.0 as i32;
                }
            } else {
                panic!("clone: ptid is not mapped")
            }
        }

        let ctid_value = if flag.contains(CloneFlags::CLONE_CHILD_SETTID)
            || flag.contains(CloneFlags::CLONE_CHILD_CLEARTID)
        {
            tid.0
        } else {
            0
        };

        if flag.contains(CloneFlags::CLONE_CHILD_SETTID)
            || flag.contains(CloneFlags::CLONE_CHILD_CLEARTID)
        {
            // TODO!(may be not map when cow fork)
            let (phy, ..) = address_space.lock().query(VirtAddr::from(ctid)).unwrap();
            unsafe {
                *(phy.as_usize() as *mut i32) = ctid_value as i32;
            }
        }
        if stack != 0 {
            assert!(flag.contains(CloneFlags::CLONE_VM));
            // set the sp of the new process
            trap_context.regs()[2] = stack;
        }

        warn!("create task pid:{}, tid:{}", pid, tid.0);
        let task = Task {
            tid,
            kernel_stack: k_stack,
            pid,
            inner: Mutex::new(TaskInner {
                name: inner.name.clone(),
                threads: MinimalManager::new(MAX_THREAD_NUM),
                thread_number: thread_num,
                address_space,
                state: TaskState::Ready,
                parent,
                children: Vec::new(),
                fd_table,
                context: Context::new(trap_return as usize, k_stack_top),
                fs_info: inner.fs_info.clone(),
                statistical_data: StatisticalData::new(),
                timer: TaskTimer::default(),
                exit_code: 0,
                heap,
                mmap: inner.mmap.clone(),
                signal_handlers,
                signal_receivers,
                set_child_tid: if flag.contains(CloneFlags::CLONE_CHILD_SETTID) {
                    ctid
                } else {
                    0
                },
                clear_child_tid: if flag.contains(CloneFlags::CLONE_CHILD_CLEARTID) {
                    ctid
                } else {
                    0
                },
                trap_cx_before_signal: None,
                signal_set_siginfo: false,
                robust: RobustList::default(),
                shm: inner.shm.clone(),
                cpu_affinity: {
                    let mut affinity = 0;
                    affinity.set_bits(0..CPU_NUM, 1 << CPU_NUM - 1);
                    affinity
                },
                unmask: 0o022,
                stack: inner.stack.clone(),
                need_wait: 0,
            }),
            send_sigchld_when_exit: sig == SignalNumber::SIGCHLD,
        };
        let task = Arc::new(task);
        if !flag.contains(CloneFlags::CLONE_PARENT) {
            inner.children.push(task.clone());
        }
        error!("create a task success");
        Some(task)
    }

    /// 用于执行一个可执行文件，供sys_exec调用。
    ///
    /// `name`用于传入文件的路径和文件名。
    /// `elf_data`用于传入从对应文件处读入的文件数据，用于构造elf_info。
    /// `args`用于指明启动可执行文件时要传入的参数。
    /// `env`用于指明相关环境变量。
    ///
    /// 成功执行则返回OK(())；否则返回错误码(isize)。
    pub fn exec(
        &self,
        name: &str,
        elf_data: &[u8],
        args: Vec<String>,
        env: Vec<String>,
    ) -> Result<(), isize> {
        let mut args = args;
        let elf_info = build_elf_address_space(elf_data, &mut args, name);
        if elf_info.is_err() {
            return Err(-1);
        }
        let elf_info = elf_info.unwrap();
        let mut inner = self.inner.lock();
        assert_eq!(inner.thread_number, 0);
        let name = elf_info.name;
        let address_space = elf_info.address_space;
        // reset the address space
        inner.address_space = Arc::new(Mutex::new(address_space));
        // reset the heap
        inner.heap = Arc::new(Mutex::new(HeapInfo::new(
            elf_info.heap_bottom,
            elf_info.heap_bottom,
        )));
        // reset the mmap
        inner.mmap = MMapInfo::new();
        // set the name of the process
        inner.name = name.to_string();
        // reset time record
        inner.statistical_data.clear();
        // close file which contains FD_CLOEXEC flag
        // now we delete all fd
        // inner.fd_table =
        // reset signal handler
        inner.signal_handlers.lock().clear();
        inner.signal_receivers.lock().clear();
        inner.timer.clear();
        inner.stack = elf_info.stack_top - USER_STACK_SIZE..elf_info.stack_top;
        let env = if env.is_empty() {
            let envp = vec![
                "LD_LIBRARY_PATH=/:/tests:/bin",
                "PS1=\x1b[1m\x1b[32mAlien\x1b[0m:\x1b[1m\x1b[34m\\w\x1b[0m\\$ \0",
                "PATH=/bin:/sbin:/usr/bin:/tests",
                "UB_BINDIR=./",
            ]
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
            envp
        } else {
            env
        };
        // we need make sure the args and env size is less than 4KB
        let phy_button = inner.transfer_raw(elf_info.stack_top - FRAME_SIZE);
        let mut user_stack = UserStack::new(phy_button + FRAME_SIZE, elf_info.stack_top);
        // push env to the top of stack of the process
        // we have push '\0' into the env string,so we don't need to push it again
        let envv = env
            .iter()
            .rev()
            .map(|env| user_stack.push_str(env).unwrap())
            .collect::<Vec<usize>>();
        // push the args to the top of stack of the process
        // we have push '\0' into the arg string,so we don't need to push it again
        let argcv = args
            .iter()
            .rev()
            .map(|arg| user_stack.push_str(arg).unwrap())
            .collect::<Vec<usize>>();
        // push padding to the top of stack of the process
        user_stack.align_to(8).unwrap();
        let random_ptr = user_stack.push_bytes(&[0u8; 16]).unwrap();
        // padding
        user_stack.push_bytes(&[0u8; 8]).unwrap();
        // push aux
        let platform = user_stack.push_str("riscv").unwrap();

        let ex_path = user_stack.push_str(&name).unwrap();
        user_stack.push(0).unwrap();
        user_stack.push(platform).unwrap();
        user_stack.push(AT_PLATFORM).unwrap();
        user_stack.push(ex_path).unwrap();
        user_stack.push(AT_EXECFN).unwrap();
        user_stack.push(elf_info.ph_num).unwrap();
        user_stack.push(AT_PHNUM).unwrap();
        user_stack.push(FRAME_SIZE).unwrap();
        user_stack.push(AT_PAGESZ).unwrap();

        user_stack.push(elf_info.bias).unwrap();
        user_stack.push(AT_BASE).unwrap();
        user_stack.push(elf_info.entry).unwrap();
        user_stack.push(AT_ENTRY).unwrap();
        user_stack.push(elf_info.ph_entry_size).unwrap();
        user_stack.push(AT_PHENT).unwrap();
        user_stack.push(elf_info.ph_drift).unwrap();
        user_stack.push(AT_PHDR).unwrap();
        user_stack.push(0).unwrap();
        user_stack.push(AT_GID).unwrap();
        user_stack.push(0).unwrap();
        user_stack.push(AT_EGID).unwrap();
        user_stack.push(0).unwrap();
        user_stack.push(AT_UID).unwrap();
        user_stack.push(0).unwrap();
        user_stack.push(AT_EUID).unwrap();
        user_stack.push(0).unwrap();
        user_stack.push(AT_SECURE).unwrap();
        user_stack.push(random_ptr).unwrap();
        user_stack.push(AT_RANDOM).unwrap();

        user_stack.push(0).unwrap();
        // push the env addr to the top of stack of the process
        envv.iter().for_each(|env| {
            user_stack.push(*env).unwrap();
        });
        user_stack.push(0).unwrap();
        // push the args addr to the top of stack of the process
        argcv.iter().enumerate().for_each(|(_i, arg)| {
            user_stack.push(*arg).unwrap();
        });
        // push the argc to the top of stack of the process
        let argc = args.len();
        let argc_ptr = user_stack.push(argc).unwrap();
        let user_sp = argc_ptr;
        warn!("args:{:?}, env:{:?}, user_sp: {:#x}", args, env, user_sp);
        let (physical, _, _) = inner
            .address_space
            .lock()
            .query(VirtAddr::from(TRAP_CONTEXT_BASE))
            .unwrap();
        let trap_frame = TrapFrame::from_raw_ptr(physical.as_usize() as *mut TrapFrame);
        *trap_frame = TrapFrame::init_for_task(
            elf_info.entry,
            user_sp,
            kernel_satp(),
            self.kernel_stack.top(),
            user_trap_vector as usize,
        );
        trap_frame.regs()[4] = elf_info.tls; // tp --> tls
        Ok(())
    }
}
