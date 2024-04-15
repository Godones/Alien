use alloc::{
    boxed::Box,
    collections::BTreeMap,
    string::{String, ToString},
    sync::{Arc, Weak},
    vec,
    vec::Vec,
};
use core::{fmt::Debug, ops::Range};

use basic::{
    sync::{Mutex, MutexGuard},
    task::{TaskContext, TaskContextExt, TrapFrame},
    vm::frame::FrameTracker,
};
use config::*;
use constants::{signal::SignalNumber, task::CloneFlags, AlienResult};
use interface::{InodeID, VFS_ROOT_ID};
use memory_addr::{PhysAddr, VirtAddr};
use page_table::MappingFlags;
use pod::Pod;
use ptable::{PhysPage, VmArea, VmAreaType, VmIo, VmSpace};
use small_index::IndexAllocator;
use task_meta::{TaskMeta, TaskStatus};

use crate::{
    elf::{build_vm_space, clone_vm_space, extend_thread_vm_space, VmmPageAllocator},
    resource::{AuxVec, FdManager, HeapInfo, KStack, TidHandle, UserStack},
    vfs_shim::{ShimFile, STDIN, STDOUT},
};

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
    /// 地址空间
    pub address_space: Arc<Mutex<VmSpace<VmmPageAllocator>>>,
    /// 文件描述符表
    pub fd_table: Arc<Mutex<FdManager>>,
    /// 堆空间
    pub heap: Arc<Mutex<HeapInfo>>,
    /// 线程计数器，用于分配同一个线程组中的线程序号
    pub threads: Arc<Mutex<IndexAllocator<MAX_THREAD_NUM>>>,
    /// 更详细的信息
    pub inner: Mutex<TaskInner>,
}

#[derive(Debug)]
pub struct TaskInner {
    /// 任务名，一般为其文件路径加文件名
    pub name: String,
    /// 用于记录当前线程在线程组中的序号
    pub thread_number: usize,
    /// 线程状态
    pub status: TaskStatus,
    /// 父亲任务控制块
    pub parent: Option<Weak<Task>>,
    /// 孩子任务控制块的集合
    pub children: BTreeMap<usize, Arc<Task>>,
    /// 任务上下文
    pub context: TaskContext,
    /// 文件系统的信息
    pub fs_info: FsContext,
    /// 返回值
    pub exit_code: i32,
    /// 子线程初始化时，将这个地址清空；子线程退出时，触发这里的 futex。
    /// 在创建时包含 CLONE_CHILD_SETTID 时才非0，但可以被 sys_set_tid_address 修改
    pub clear_child_tid: usize,
    /// 栈空间的信息
    pub stack: Range<usize>,
}

#[derive(Debug, Clone, Copy)]
pub struct FsContext {
    /// current working directory
    pub cwd: InodeID,
    /// root directory
    pub root: InodeID,
}

impl FsContext {
    pub fn new(cwd: InodeID, root: InodeID) -> Self {
        Self { cwd, root }
    }
}

impl Task {
    pub fn pid(&self) -> usize {
        self.pid.raw()
    }

    pub fn tid(&self) -> usize {
        self.tid.raw()
    }

    pub fn exit_code(&self) -> i32 {
        self.inner.lock().exit_code
    }

    pub fn inner(&self) -> MutexGuard<TaskInner> {
        self.inner.lock()
    }

    pub fn status(&self) -> TaskStatus {
        let inner = self.inner.lock();
        inner.status
    }

    pub fn create_task_meta(&self) -> TaskMeta {
        TaskMeta::new(self.tid(), self.inner().context)
    }
    pub fn set_tid_address(&self, addr: usize) {
        let mut inner = self.inner.lock();
        inner.clear_child_tid = addr;
    }
    pub fn get_file(&self, fd: usize) -> Option<Arc<ShimFile>> {
        self.fd_table.lock().get(fd)
    }

    pub fn add_file(&self, file: Arc<ShimFile>) -> usize {
        self.fd_table.lock().insert(file)
    }

    pub fn token(&self) -> usize {
        let paddr = self.address_space.lock().root_paddr();
        (8usize << 60) | (paddr >> 12)
    }

    pub fn read_bytes_from_user(&self, src: VirtAddr, dest: &mut [u8]) -> AlienResult<()> {
        let vm_space = self.address_space.lock();
        vm_space.read_bytes(src, dest).unwrap();
        Ok(())
    }

    pub fn read_val_from_user<T: Pod>(&self, src: VirtAddr) -> AlienResult<T> {
        let vm_space = self.address_space.lock();
        let val = vm_space.read_val(src).unwrap();
        Ok(val)
    }

    pub fn write_bytes_to_user(&self, dest: VirtAddr, src: &[u8]) -> AlienResult<()> {
        let mut vm_space = self.address_space.lock();
        vm_space.write_bytes(dest, src).unwrap();
        Ok(())
    }

    pub fn write_val_to_user<T: Pod>(&self, dest: VirtAddr, val: &T) -> AlienResult<()> {
        let mut vm_space = self.address_space.lock();
        vm_space.write_val(dest, val).unwrap();
        Ok(())
    }

    pub fn read_string_from_user(&self, src: VirtAddr) -> AlienResult<String> {
        let mut s = Vec::with_capacity(128);
        let mut ptr = src;
        loop {
            let c = self.read_val_from_user::<u8>(ptr)?;
            if c == 0 {
                break;
            }
            s.push(c);
            ptr += core::mem::size_of::<u8>();
        }
        let str = String::from_utf8(s).unwrap();
        Ok(str)
    }

    pub fn trap_frame(&self) -> &'static mut TrapFrame {
        TrapFrame::from_raw_phy_ptr(self.trap_frame_phy_ptr())
    }

    pub fn trap_frame_virt_ptr(&self) -> VirtAddr {
        let inner = self.inner();
        let trap_context_base = TRAP_CONTEXT_BASE - inner.thread_number * FRAME_SIZE;
        VirtAddr::from(trap_context_base)
    }

    pub fn trap_frame_phy_ptr(&self) -> PhysAddr {
        let trap_context_base = TRAP_CONTEXT_BASE - self.inner().thread_number * FRAME_SIZE;
        let (physical, _, _) = self.address_space.lock().query(trap_context_base).unwrap();
        physical
    }

    pub fn extend_heap(&self, addr: usize) -> usize {
        let mut heap = self.heap.lock();
        heap.current = addr;
        if addr < heap.end {
            return heap.current;
        }
        let addition = addr - heap.end;
        // increase heap size
        let end = heap.end;
        // align addition to PAGE_SIZE
        let addition = (addition + FRAME_SIZE - 1) & !(FRAME_SIZE - 1);
        warn!(
            "extend heap: {:#x} -> {:#x}, addition: {:#x}",
            end,
            end + addition,
            addition
        );
        let mut phy_frames: Vec<Box<dyn PhysPage>> = vec![];
        for _ in 0..addition / FRAME_SIZE {
            let page = FrameTracker::new(1);
            phy_frames.push(Box::new(page));
        }
        let area = VmArea::new(
            end..end + addition,
            MappingFlags::READ | MappingFlags::WRITE | MappingFlags::USER,
            phy_frames,
        );
        let mut guard = self.address_space.lock();
        guard.map(VmAreaType::VmArea(area)).unwrap();
        heap.end = end + addition;
        heap.current
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
        let stack_info =
            elf_info.stack_top.as_usize() - USER_STACK_SIZE..elf_info.stack_top.as_usize();
        let task = Task {
            tid,
            kernel_stack: k_stack,
            pid,
            address_space: Arc::new(Mutex::new(address_space)),
            fd_table: {
                let mut fd_table = FdManager::new();
                fd_table.insert(STDIN.clone());
                fd_table.insert(STDOUT.clone());
                fd_table.insert(STDOUT.clone());
                Arc::new(Mutex::new(fd_table))
            },
            threads: {
                let mut allocator = IndexAllocator::new();
                let number = allocator.allocate().unwrap();
                assert_eq!(number, 0);
                Arc::new(Mutex::new(allocator))
            },
            heap: Arc::new(Mutex::new(HeapInfo::new(
                elf_info.heap_bottom.as_usize(),
                elf_info.heap_bottom.as_usize(),
            ))),
            inner: Mutex::new(TaskInner {
                name: name.to_string(),
                thread_number: 0,
                status: TaskStatus::Ready,
                parent: None,
                children: BTreeMap::new(),
                context: TaskContext::new_user(k_stack_top),
                fs_info: FsContext::new(VFS_ROOT_ID, VFS_ROOT_ID),
                exit_code: 0,
                clear_child_tid: 0,
                stack: stack_info,
            }),
            send_sigchld_when_exit: false,
        };
        let mut user_stack = UserStack::new(
            elf_info.stack_top,
            vec![],
            vec![],
            AuxVec::default(),
            name.to_string(),
        );
        let user_sp = user_stack.init(&mut task.address_space.lock()).unwrap();
        let trap_frame = task.trap_frame();
        *trap_frame = TrapFrame::new_user(elf_info.entry, user_sp, k_stack_top);
        trap_frame.update_tp(VirtAddr::from(elf_info.tls)); // tp --> tls
        Some(task)
    }

    pub fn do_clone(self: &Arc<Self>, clone_args: CloneArgs) -> Option<Arc<Task>> {
        info!("<do_clone> args: {:?}", clone_args);
        let tid = Arc::new(TidHandle::new()?);
        let address_space = if clone_args.flags.contains(CloneFlags::CLONE_VM) {
            // create thread
            self.address_space.clone()
        } else {
            // create sub process
            let address_space = clone_vm_space(&self.address_space.lock());
            Arc::new(Mutex::new(address_space))
        };
        let inner = self.inner.lock();

        // map the thread trap_context if clone_vm
        let thread_num = if clone_args.flags.contains(CloneFlags::CLONE_VM) {
            let thread_num = self.threads.lock().allocate().unwrap();
            info!("thread_num: {}", thread_num);
            // calculate the address for thread context
            extend_thread_vm_space(&mut address_space.lock(), thread_num);
            thread_num
        } else {
            assert_eq!(inner.thread_number, 0);
            inner.thread_number
        };
        let parent = if clone_args.flags.contains(CloneFlags::CLONE_PARENT) {
            inner.parent.clone()
        } else {
            Some(Arc::downgrade(self))
        };

        let (name, fs_info, stack) = (
            inner.name.clone(),
            inner.fs_info.clone(),
            inner.stack.clone(),
        );

        drop(inner);

        let fd_table = if clone_args.flags.contains(CloneFlags::CLONE_FILES) {
            self.fd_table.clone()
        } else {
            Arc::new(Mutex::new(self.fd_table.lock().clone()))
        };

        let threads = if clone_args.flags.contains(CloneFlags::CLONE_THREAD) {
            self.threads.clone()
        } else {
            let mut allocator = IndexAllocator::new();
            let number = allocator.allocate().unwrap();
            assert_eq!(number, 0);
            Arc::new(Mutex::new(allocator))
        };

        let pid = if clone_args.flags.contains(CloneFlags::CLONE_THREAD) {
            self.pid.clone()
        } else {
            tid.clone()
        };

        let heap = if clone_args.flags.contains(CloneFlags::CLONE_VM) {
            self.heap.clone()
        } else {
            Arc::new(Mutex::new(self.heap.lock().clone()))
        };

        let k_stack = KStack::new(USER_KERNEL_STACK_SIZE / FRAME_SIZE);
        let k_stack_top = k_stack.top();

        info!("create task pid:{:?}, tid:{:?}", pid, tid);
        let task = Task {
            tid: tid.clone(),
            kernel_stack: k_stack,
            pid,
            threads,
            address_space,
            fd_table,
            heap,
            inner: Mutex::new(TaskInner {
                name,
                thread_number: thread_num,
                status: TaskStatus::Ready,
                parent,
                children: BTreeMap::new(),
                context: TaskContext::new_user(k_stack_top),
                fs_info,
                exit_code: 0,
                clear_child_tid: if clone_args.flags.contains(CloneFlags::CLONE_CHILD_CLEARTID) {
                    clone_args.ctid
                } else {
                    0
                },
                stack,
            }),
            send_sigchld_when_exit: clone_args.sig == SignalNumber::SIGCHLD,
        };

        // let old_trap_context = self.trap_frame();
        let trap_context = task.trap_frame();

        // *trap_context = *old_trap_context;
        // 设置内核栈地址
        trap_context.update_k_sp(k_stack_top);

        // 检查是否需要设置 tls
        if clone_args.flags.contains(CloneFlags::CLONE_SETTLS) {
            trap_context.update_tp(VirtAddr::from(clone_args.tls));
        }

        // 检查是否在父任务地址中写入 tid
        if clone_args.flags.contains(CloneFlags::CLONE_PARENT_SETTID) {
            task.write_val_to_user(VirtAddr::from(clone_args.ptid), &(tid.raw() as i32))
                .unwrap();
        }

        if clone_args.flags.contains(CloneFlags::CLONE_CHILD_SETTID) {
            task.write_val_to_user(VirtAddr::from(clone_args.ctid), &(tid.raw() as i32))
                .unwrap();
        }

        if clone_args.stack != 0 {
            assert!(clone_args.flags.contains(CloneFlags::CLONE_VM));
            // set the sp of the new process
            trap_context.update_user_sp(VirtAddr::from(clone_args.stack))
        }

        let task = Arc::new(task);
        if !clone_args.flags.contains(CloneFlags::CLONE_PARENT) {
            self.inner.lock().children.insert(task.pid(), task.clone());
        }
        info!("create a task success");
        Some(task)
    }

    pub fn do_execve(
        &self,
        name: &str,
        elf_data: &[u8],
        mut argv: Vec<String>,
        envp: Vec<String>,
    ) -> AlienResult<()> {
        let elf_info = build_vm_space(elf_data, &mut argv, name)?;
        let aux = AuxVec::from_elf_info(&elf_info)?;
        let mut inner = self.inner.lock();
        assert_eq!(inner.thread_number, 0);
        let address_space = elf_info.address_space;
        // reset the address space
        *self.address_space.lock() = address_space;
        // reset the heap
        *self.heap.lock() = HeapInfo::new(
            elf_info.heap_bottom.as_usize(),
            elf_info.heap_bottom.as_usize(),
        );
        // set the name of the process
        inner.name = elf_info.name;
        // close file which contains FD_CLOEXEC flag
        // now we delete all fd
        // reset signal handler
        inner.stack =
            elf_info.stack_top.as_usize() - USER_STACK_SIZE..elf_info.stack_top.as_usize();
        info!("argv:{:?}, env:{:?}", argv, envp);
        let mut user_stack = UserStack::new(elf_info.stack_top, argv, envp, aux, name.to_string());
        let user_sp = user_stack.init(&mut self.address_space.lock())?;
        info!("user_sp: {:#x}", user_sp);
        drop(inner);
        let trap_frame = self.trap_frame();

        *trap_frame = TrapFrame::new_user(elf_info.entry, user_sp, self.kernel_stack.top());
        trap_frame.update_tp(VirtAddr::from(elf_info.tls)); // tp --> tls
        Ok(())
    }
}

#[derive(Debug)]
pub struct CloneArgs {
    pub flags: CloneFlags,
    pub sig: SignalNumber,
    pub stack: usize,
    pub ptid: usize,
    pub tls: usize,
    pub ctid: usize,
}
