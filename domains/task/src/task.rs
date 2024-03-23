use crate::elf::{
    build_vm_space, clone_vm_space, extend_thread_vm_space, FrameTracker, VmmPageAllocator,
};
use crate::resource::{FdManager, HeapInfo, KStack, TidHandle, UserStack};
use crate::vfs_shim::{ShimFile, STDIN, STDOUT};
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use alloc::{format, vec};
use config::{
    FRAME_SIZE, MAX_THREAD_NUM, TRAP_CONTEXT_BASE, USER_KERNEL_STACK_SIZE, USER_STACK_SIZE,
};
use constants::aux::*;
use constants::signal::SignalNumber;
use constants::task::CloneFlags;
use constants::AlienResult;
use context::{TaskContext, TrapFrame};
use core::fmt::Debug;
use core::ops::Range;
use interface::{InodeId, VFS_ROOT_ID};
use ksync::{Mutex, MutexGuard};
use ptable::{MappingFlags, PagingIf, PhyFrame, VmArea, VmAreaType, VmSpace};
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
pub enum TaskStatus {
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

    pub fn update_state(&self, state: TaskStatus) {
        let mut inner = self.inner.lock();
        inner.status = state;
    }

    pub fn get_file(&self, fd: usize) -> Option<Arc<ShimFile>> {
        self.fd_table.lock().get(fd)
    }

    pub fn token(&self) -> usize {
        let paddr = self.address_space.lock().root_paddr();
        (8usize << 60) | (paddr >> 12)
    }

    pub fn transfer_raw(&self, ptr: usize) -> usize {
        let guard = self.address_space.lock();
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
        let (phy_addr, _, _) = self.address_space.lock().query(ptr).unwrap();
        let phy_addr = phy_addr.as_usize();
        let ptr = phy_addr as *mut T;
        unsafe { &mut *ptr }
    }

    pub fn trap_frame(&self) -> &'static mut TrapFrame {
        let trap_context_base = TRAP_CONTEXT_BASE - self.inner().thread_number * FRAME_SIZE;
        let (physical, _, _) = self.address_space.lock().query(trap_context_base).unwrap();
        TrapFrame::from_raw_ptr(physical.as_usize() as *mut TrapFrame)
    }

    pub fn trap_frame_ptr(&self) -> usize {
        let inner = self.inner();
        let trap_context_base = TRAP_CONTEXT_BASE - inner.thread_number * FRAME_SIZE;
        trap_context_base
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
        let mut phy_frames = vec![];
        for _ in 0..addition / FRAME_SIZE {
            let page = VmmPageAllocator::alloc_frame().unwrap();
            phy_frames.push(PhyFrame::new(Box::new(FrameTracker::from_addr(
                page.as_usize(),
                1,
                true,
            ))));
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

    pub fn copy_to_user(&self, src: *const u8, dst: *mut u8, len: usize) {
        let buf = self.transfer_buffer(dst, len);
        let mut start = 0;
        let mut src = src as usize;
        for b in buf {
            let len = if start + b.len() > len {
                len - start
            } else {
                b.len()
            };
            unsafe {
                core::ptr::copy_nonoverlapping(src as _, b.as_mut_ptr(), len);
            }
            start += len;
            src += len;
        }
    }

    pub fn copy_from_user(&self, src: *const u8, dst: *mut u8, len: usize) {
        let buf = self.transfer_buffer(src, len);
        let mut start = 0;
        let mut dst = dst as usize;
        for b in buf {
            let len = if start + b.len() > len {
                len - start
            } else {
                b.len()
            };
            unsafe {
                core::ptr::copy_nonoverlapping(b.as_ptr(), dst as _, len);
            }
            start += len;
            dst += len;
        }
    }

    pub fn read_string_from_user(&self, ptr: usize) -> String {
        let mut s = Vec::new();
        let mut ptr = ptr;
        loop {
            let ptr_value = self.transfer_raw_ptr(ptr as *mut u8);
            if *ptr_value == 0 {
                break;
            }
            s.push(*ptr_value);
            ptr += 1;
        }
        String::from_utf8(s).unwrap()
    }

    fn transfer_buffer<T: Debug>(&self, ptr: *const T, len: usize) -> Vec<&'static mut [T]> {
        let mut start = ptr as usize;
        let end = start + len;
        let mut v = Vec::new();

        let guard = self.address_space.lock();

        while start < end {
            let (start_phy, _flag, _) = guard
                .query(start)
                .expect(format!("transfer_buffer: {:x} failed", start).as_str());
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

        let trap_to_user = basic::trap_to_user();

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
                elf_info.heap_bottom,
                elf_info.heap_bottom,
            ))),
            inner: Mutex::new(TaskInner {
                name: name.to_string(),
                thread_number: 0,
                status: TaskStatus::Ready,
                parent: None,
                children: BTreeMap::new(),
                context: TaskContext::new(trap_to_user, k_stack_top),
                fs_info: FsContext::new(VFS_ROOT_ID, VFS_ROOT_ID),
                exit_code: 0,
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
        let kernel_satp = basic::kernel_satp();
        let user_trap_vector = basic::trap_from_user();

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

        let trap_to_user = basic::trap_to_user();
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
                context: TaskContext::new(trap_to_user, k_stack_top),
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
        trap_context.update_kernel_sp(k_stack_top);

        // 检查是否需要设置 tls
        if clone_args.flags.contains(CloneFlags::CLONE_SETTLS) {
            trap_context.update_tp(clone_args.tls);
        }

        // 检查是否在父任务地址中写入 tid
        if clone_args.flags.contains(CloneFlags::CLONE_PARENT_SETTID) {
            *task.transfer_raw_ptr(clone_args.ptid as *mut i32) = tid.raw() as i32;
        }

        if clone_args.flags.contains(CloneFlags::CLONE_CHILD_SETTID) {
            *task.transfer_raw_ptr(clone_args.ctid as *mut i32) = tid.raw() as _;
        }

        if clone_args.stack != 0 {
            assert!(clone_args.flags.contains(CloneFlags::CLONE_VM));
            // set the sp of the new process
            trap_context.update_user_sp(clone_args.stack)
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
        let mut inner = self.inner.lock();
        assert_eq!(inner.thread_number, 0);
        let address_space = elf_info.address_space;
        // reset the address space
        // self.address_space = Arc::new(Mutex::new(address_space));
        *self.address_space.lock() = address_space;
        // reset the heap
        *self.heap.lock() = HeapInfo::new(elf_info.heap_bottom, elf_info.heap_bottom);
        // set the name of the process
        inner.name = elf_info.name;
        // close file which contains FD_CLOEXEC flag
        // now we delete all fd

        // reset signal handler
        inner.stack = elf_info.stack_top - USER_STACK_SIZE..elf_info.stack_top;

        let phy_button = self.transfer_raw(elf_info.stack_top - FRAME_SIZE);
        let mut user_stack = UserStack::new(phy_button + FRAME_SIZE, elf_info.stack_top);

        // push env to the top of stack of the process
        // we have push '\0' into the env string,so we don't need to push it again
        let envv = envp
            .iter()
            .rev()
            .map(|env| user_stack.push_str(env).unwrap())
            .collect::<Vec<usize>>();
        // push the args to the top of stack of the process
        // we have push '\0' into the arg string,so we don't need to push it again
        let argcv = argv
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
        let argc = argv.len();
        let argc_ptr = user_stack.push(argc).unwrap();
        let user_sp = argc_ptr;
        warn!("args:{:?}, env:{:?}, user_sp: {:#x}", argv, envp, user_sp);
        drop(inner);
        let trap_frame = self.trap_frame();

        let kernel_satp = basic::kernel_satp();
        let user_trap_vector = basic::trap_from_user();

        *trap_frame = TrapFrame::init_for_task(
            elf_info.entry,
            user_sp,
            kernel_satp,
            self.kernel_stack.top(),
            user_trap_vector,
        );
        trap_frame.regs()[4] = elf_info.tls; // tp --> tls
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
