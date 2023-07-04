use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;

use lazy_static::lazy_static;
use page_table::addr::{align_down_4k, VirtAddr};
use page_table::pte::MappingFlags;
use page_table::table::Sv39PageTable;
use rvfs::dentry::DirEntry;
use rvfs::file::File;
use rvfs::info::ProcessFsInfo;
use rvfs::mount::VfsMount;

use gmanager::MinimalManager;
use kernel_sync::{Mutex, MutexGuard};
use syscall_define::aux::{
    AT_EGID, AT_ENTRY, AT_EUID, AT_EXECFN, AT_GID, AT_PAGESZ, AT_PHDR, AT_PHENT, AT_PHNUM,
    AT_PLATFORM, AT_RANDOM, AT_SECURE, AT_UID,
};
use syscall_define::signal::{SignalHandlers, SignalReceivers};

use crate::config::{FRAME_BITS, MAX_FD_NUM, TRAP_CONTEXT_BASE, USER_STACK_SIZE};
use crate::config::{FRAME_SIZE, MAX_THREAD_NUM, USER_KERNEL_STACK_SIZE};
use crate::error::{AlienError, AlienResult};
use crate::fs::{STDIN, STDOUT};
use crate::memory::{
    build_clone_address_space, build_elf_address_space, kernel_satp, MMapInfo, MMapRegion,
    MapFlags, PageAllocator, ProtFlags, UserStack, FRAME_REF_MANAGER,
};
use crate::task::context::Context;
use crate::task::cpu::{CloneFlags, SignalFlags};
use crate::task::heap::HeapInfo;
use crate::task::stack::Stack;
use crate::timer::read_timer;
use crate::trap::{trap_return, user_trap_vector, TrapFrame};

type FdManager = MinimalManager<Arc<File>>;

lazy_static! {
    /// 这里把MinimalManager复用为pid分配器，通常，MinimalManager会将数据插入到最小可用位置并返回位置，
    /// 但pid的分配并不需要实际存储信息，因此可以插入任意的数据，这里为了节省空间，将数据定义为u8
    pub static ref TID_MANAGER:Mutex<MinimalManager<u8>> = Mutex::new(MinimalManager::new(MAX_THREAD_NUM));
}
#[derive(Debug)]
pub struct TidHandle(usize);

impl TidHandle {
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
    pub tid: TidHandle,
    pub pid: usize,
    /// 当退出时是否向父进程发送信号 SIGCHLD。
    /// 如果创建时带 CLONE_THREAD 选项，则不发送信号，除非它是线程组(即拥有相同pid的所有线程)中最后一个退出的线程；
    /// 否则发送信号
    pub send_sigchld_when_exit: bool,
    pub kernel_stack: Stack,
    inner: Mutex<TaskInner>,
}

#[derive(Debug)]
pub struct TaskInner {
    pub name: String,
    pub address_space: Arc<Mutex<Sv39PageTable<PageAllocator>>>,
    pub state: TaskState,
    pub parent: Option<Weak<Task>>,
    pub children: Vec<Arc<Task>>,
    pub fd_table: FdManager,
    pub context: Context,
    pub fs_info: FsContext,
    pub statistical_data: StatisticalData,
    pub exit_code: i32,
    pub heap: HeapInfo,
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
    trap_cx_before_signal: Option<TrapFrame>,
    /// 保存信息时，处理函数是否设置了 SIGINFO 选项
    /// 如果设置了，说明信号触发前的上下文信息通过 ucontext 传递给了用户，
    /// 此时用户可能修改其中的 pc 信息(如musl-libc 的 pthread_cancel 函数)。
    /// 在这种情况下，需要手动在 sigreturn 时更新已保存的上下文信息
    signal_set_siginfo: bool,
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

#[derive(Clone, Debug)]
pub struct FsContext {
    /// 当前工作目录
    pub cwd: Arc<DirEntry>,
    /// 根目录
    pub root: Arc<DirEntry>,
    /// 当前挂载点
    pub cmnt: Arc<VfsMount>,
    /// 根挂载点
    pub rmnt: Arc<VfsMount>,
}

impl FsContext {
    pub fn empty() -> Self {
        FsContext {
            cwd: Arc::new(DirEntry::empty()),
            root: Arc::new(DirEntry::empty()),
            cmnt: Arc::new(VfsMount::empty()),
            rmnt: Arc::new(VfsMount::empty()),
        }
    }

    pub fn new(
        root: Arc<DirEntry>,
        cwd: Arc<DirEntry>,
        cmnt: Arc<VfsMount>,
        rmnt: Arc<VfsMount>,
    ) -> Self {
        FsContext {
            cwd,
            root,
            cmnt,
            rmnt,
        }
    }
}

impl Into<ProcessFsInfo> for FsContext {
    fn into(self) -> ProcessFsInfo {
        ProcessFsInfo {
            root_mount: self.rmnt.clone(),
            root_dir: self.root.clone(),
            current_dir: self.cwd.clone(),
            current_mount: self.cmnt.clone(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub enum TaskState {
    Ready,
    Running,
    // waiting for some time
    Sleeping,
    // waiting some event
    Waiting,
    // waiting for parent to reap
    Zombie,
    // terminated
    Terminated,
}

impl Task {
    #[inline]
    pub fn get_pid(&self) -> isize {
        self.pid as isize
    }
    #[inline]
    pub fn get_tid(&self) -> isize {
        self.tid.0 as isize
    }

    pub fn set_tid_address(&self, tidptr: usize) {
        let mut inner = self.inner.lock();
        inner.clear_child_tid = tidptr;
    }
    pub fn get_name(&self) -> String {
        let inner = self.inner.lock();
        inner.name.clone()
    }
    pub fn access_inner(&self) -> MutexGuard<TaskInner> {
        self.inner.lock()
    }
    pub fn token(&self) -> usize {
        let inner = self.inner.lock();
        let paddr = inner.address_space.lock().root_paddr();
        (8usize << 60) | (paddr.as_usize() >> 12)
    }

    pub fn trap_frame(&self) -> &'static mut TrapFrame {
        let inner = self.inner.lock();
        let (physical, _, _) = inner
            .address_space
            .lock()
            .query(VirtAddr::from(TRAP_CONTEXT_BASE))
            .unwrap();
        TrapFrame::from_raw_ptr(physical.as_usize() as *mut TrapFrame)
    }

    pub fn update_state(&self, state: TaskState) {
        let mut inner = self.inner.lock();
        inner.state = state;
    }

    pub fn state(&self) -> TaskState {
        let inner = self.inner.lock();
        inner.state
    }

    pub fn update_exit_code(&self, code: i32) {
        let mut inner = self.inner.lock();
        inner.exit_code = code;
    }

    pub fn get_context_raw_ptr(&self) -> *const Context {
        let inner = self.inner.lock();
        &inner.context as *const Context
    }
    pub fn get_context_mut_raw_ptr(&self) -> *mut Context {
        let mut inner = self.inner.lock();
        &mut inner.context as *mut Context
    }

    pub fn children(&self) -> Vec<Arc<Task>> {
        let inner = self.inner.lock();
        inner.children.clone()
    }
    pub fn remove_child(&self, index: usize) -> Arc<Task> {
        let mut inner = self.inner.lock();
        assert!(index < inner.children.len());
        inner.children.remove(index)
    }

    pub fn update_parent(&self, parent: Arc<Task>) {
        let mut inner = self.inner.lock();
        inner.parent = Some(Arc::downgrade(&parent));
    }

    pub fn insert_child(&self, child: Arc<Task>) {
        let mut inner = self.inner.lock();
        inner.children.push(child);
    }

    pub fn exit_code(&self) -> i32 {
        let inner = self.inner.lock();
        inner.exit_code
    }

    pub fn get_file(&self, fd: usize) -> Option<Arc<File>> {
        let inner = self.inner.lock();
        let file = inner.fd_table.get(fd);
        return if file.is_err() { None } else { file.unwrap() };
    }
    pub fn add_file(&self, file: Arc<File>) -> Result<usize, ()> {
        self.access_inner().fd_table.insert(file).map_err(|_| {})
    }
    pub fn add_file_with_fd(&self, file: Arc<File>, fd: usize) -> Result<(), ()> {
        let mut inner = self.access_inner();
        inner.fd_table.insert_with_index(fd, file).map_err(|_| {})
    }

    pub fn remove_file(&self, fd: usize) -> Result<Arc<File>, ()> {
        let mut inner = self.inner.lock();
        let file = inner.fd_table.get(fd);
        if file.is_err() {
            return Err(());
        }
        let file = file.unwrap();
        if file.is_none() {
            return Err(());
        }
        let file = file.unwrap();
        inner.fd_table.remove(fd).map_err(|_| {})?;
        Ok(file)
    }

    pub fn transfer_raw(&self, ptr: usize) -> usize {
        self.access_inner().transfer_raw(ptr)
    }
    // TODO 处理跨页问题
    pub fn transfer_raw_ptr<T>(&self, ptr: *mut T) -> &'static mut T {
        self.access_inner().transfer_raw_ptr(ptr)
    }
    // TODO:处理效率低的问题
    pub fn transfer_str(&self, ptr: *const u8) -> String {
        self.access_inner().transfer_str(ptr)
    }
    pub fn transfer_raw_buffer(&self, ptr: *const u8, len: usize) -> Vec<&'static mut [u8]> {
        self.access_inner().transfer_raw_buffer(ptr, len)
    }
    pub fn transfer_buffer<T>(&self, ptr: *const T, len: usize) -> Vec<&'static mut [T]> {
        self.access_inner().transfer_buffer(ptr, len)
    }
}

impl TaskInner {
    pub fn cwd(&self) -> FsContext {
        self.fs_info.clone()
    }
    pub fn transfer_raw(&self, ptr: usize) -> usize {
        let (phy, ..) = self
            .address_space
            .lock()
            .query(VirtAddr::from(ptr))
            .unwrap();
        phy.as_usize()
    }
    pub fn transfer_str(&self, ptr: *const u8) -> String {
        let mut res = String::new();
        let mut start = ptr as usize;
        loop {
            let physical = self.address_space.lock().query(VirtAddr::from(start));
            if physical.is_err() {
                break;
            }
            let (physical, _, _) = physical.unwrap();
            let c = unsafe { &*(physical.as_usize() as *const u8) };
            if *c == 0 {
                break;
            }
            res.push(*c as char);
            start += 1;
        }
        res
    }
    pub fn transfer_raw_buffer(&self, ptr: *const u8, len: usize) -> Vec<&'static mut [u8]> {
        let address_space = &self.address_space.lock();
        let mut start = ptr as usize;
        let end = start + len;
        let mut v = Vec::new();
        while start < end {
            let (start_phy, _, _) = address_space.query(VirtAddr::from(start)).unwrap();
            // start_phy向上取整到FRAME_SIZE
            let bound = (start & !(FRAME_SIZE - 1)) + FRAME_SIZE;
            let len = if bound > end {
                end - start
            } else {
                bound - start
            };
            unsafe {
                let buf = core::slice::from_raw_parts_mut(start_phy.as_usize() as *mut u8, len);
                v.push(buf);
            }
            start = bound;
        }
        v
    }

    pub fn transfer_buffer<T>(&self, ptr: *const T, len: usize) -> Vec<&'static mut [T]> {
        let address_space = &self.address_space.lock();
        let mut start = ptr as usize;
        let end = start + len;
        let mut v = Vec::new();
        while start < end {
            let (start_phy, _, _) = address_space.query(VirtAddr::from(start)).unwrap();
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

    pub fn transfer_raw_ptr<T>(&self, ptr: *mut T) -> &'static mut T {
        let (physical, _, _) = self
            .address_space
            .lock()
            .query(VirtAddr::from(ptr as usize))
            .unwrap();
        unsafe { &mut *(physical.as_usize() as *mut T) }
    }

    /// When process return to user mode, we need to update the user mode time
    /// WARNING: If the cause of the process returning to the kernel is a timer interrupt,
    /// We should not call this function.
    pub fn update_kernel_mode_time(&mut self) {
        let now = read_timer(); // current cpu clocks
        let time = now - self.statistical_data.last_stime;
        self.statistical_data.tms_stime += time;
        self.statistical_data.last_utime = now;
    }

    /// When process return to kernel mode, we need to update the user Mode Time
    pub fn update_user_mode_time(&mut self) {
        let now = read_timer(); // current cpu clocks
        let time = now - self.statistical_data.last_utime;
        self.statistical_data.tms_utime += time;
        self.statistical_data.last_stime = now;
    }

    pub fn statistical_data(&self) -> &StatisticalData {
        &self.statistical_data
    }

    pub fn heap_info(&self) -> HeapInfo {
        self.heap.clone()
    }

    pub fn shrink_heap(_addr: usize) -> Result<usize, AlienError> {
        todo!()
    }

    /// extend heap
    pub fn extend_heap(&mut self, addr: usize) -> Result<usize, AlienError> {
        self.heap.current = addr;
        if addr < self.heap.end {
            return Ok(self.heap.current);
        }
        let addition = addr - self.heap.end;
        // increase heap size
        let end = self.heap.end;
        // align addition to PAGE_SIZE
        let addition = (addition + FRAME_SIZE - 1) & !(FRAME_SIZE - 1);
        self.address_space
            .lock()
            .map_region_no_target(
                VirtAddr::from(end),
                addition,
                "RWUAD".into(), // no V flag
                true,
                true,
            )
            .unwrap();
        let new_end = end + addition;
        self.heap.end = new_end;
        Ok(self.heap.current)
    }

    /// the len will be aligned to 4k
    pub fn add_mmap(
        &mut self,
        _start: usize,
        len: usize,
        prot: ProtFlags,
        flags: MapFlags,
        fd: usize,
        offset: usize,
    ) -> Result<usize, isize> {
        let file = self.fd_table.get(fd).map_err(|_| -1isize)?;
        if file.is_none() {
            return Err(-1);
        }
        let v_range = self.mmap.alloc(len);
        let region = MMapRegion::new(
            v_range.start,
            len,
            v_range.end - v_range.start,
            prot,
            flags,
            file.unwrap(),
            offset,
        );
        // warn!("add mmap region:{:#x?}",region);
        self.mmap.add_region(region);
        let start = v_range.start;
        let mut map_flags = prot.into(); // no V  flag
        map_flags |= "AD".into();
        self.address_space
            .lock()
            .map_region_no_target(
                VirtAddr::from(start),
                v_range.end - start,
                map_flags,
                true,
                true,
            )
            .unwrap();
        Ok(start)
    }

    pub fn unmap(&mut self, start: usize, len: usize) -> Result<(), isize> {
        // check whether the start is in mmap
        let x = self.mmap.get_region(start);
        if x.is_none() {
            return Err(-1);
        }
        // now we need make sure the start is equal to the start of the region, and the len is equal to the len of the region
        let region = x.unwrap();
        if region.start != start || len != region.len {
            return Err(-1);
        }
        self.address_space
            .lock()
            .unmap_region(VirtAddr::from(start), region.map_len)
            .unwrap();
        self.mmap.remove_region(start);
        Ok(())
    }

    pub fn do_load_page_fault(
        &mut self,
        addr: usize,
    ) -> Result<(Arc<File>, &'static mut [u8], u64), isize> {
        // check whether the addr is in mmap
        let x = self.mmap.get_region(addr);
        if x.is_none() {
            return Err(-1);
        }
        // now we need make sure the start is equal to the start of the region, and the len is equal to the len of the region
        let region = x.unwrap();
        assert_eq!(addr % FRAME_SIZE, 0);
        // update page table
        let mut map_flags = region.prot.into();
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
        Ok((file.clone(), buf, read_offset as u64))
    }

    fn invalid_page_solver(
        &mut self,
        addr: usize,
    ) -> AlienResult<Option<(Arc<File>, &'static mut [u8], u64)>> {
        let is_mmap = self.mmap.get_region(addr);
        let is_heap = self.heap.contains(addr);
        if is_mmap.is_none() && !is_heap {
            return Err(AlienError::Other);
        }
        if is_heap {
            trace!("invalid page fault in heap");
            let map_flags = "RWUVAD".into();
            self.address_space
                .lock()
                .validate(VirtAddr::from(addr), map_flags)
                .unwrap();
        } else {
            let region = is_mmap.unwrap();
            assert_eq!(addr % FRAME_SIZE, 0);
            // update page table
            let mut map_flags = region.prot.into();
            map_flags |= "V".into();
            self.address_space
                .lock()
                .validate(VirtAddr::from(addr), map_flags)
                .unwrap();
            let (phy, _, size) = self
                .address_space
                .lock()
                .query(VirtAddr::from(addr))
                .unwrap();
            let buf =
                unsafe { core::slice::from_raw_parts_mut(phy.as_usize() as *mut u8, size.into()) };
            let file = &region.fd;
            let read_offset = region.offset + (addr - region.start);
            return Ok(Some((file.clone(), buf, read_offset as u64)));
        }
        Ok(None)
    }

    pub fn do_store_page_fault(
        &mut self,
        addr: usize,
    ) -> AlienResult<Option<(Arc<File>, &'static mut [u8], u64)>> {
        trace!("do store page fault:{:#x}", addr);
        let addr = align_down_4k(addr);
        let (phy, flags, page_size) = self
            .address_space
            .lock()
            .query(VirtAddr::from(addr))
            .expect(format!("addr:{:#x}", addr).as_str());
        if !flags.contains(MappingFlags::V) {
            return self.invalid_page_solver(addr);
        }
        assert!(flags.contains(MappingFlags::RSD), "flags:{:#x}", flags);
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
            core::ptr::copy_nonoverlapping(src_ptr, dst_ptr, usize::from(page_size));
        }
        for i in 0..usize::from(page_size) / FRAME_SIZE {
            let t_phy = phy + i * FRAME_SIZE;
            FRAME_REF_MANAGER
                .lock()
                .dec_ref(t_phy.as_usize() >> FRAME_BITS);
        }
        Ok(None)
    }
}

impl Task {
    pub fn recycle(&self) {
        let mut inner = self.inner.lock();
        // delete child process
        inner.children.clear();
        // recycle page
    }
    /// only call once
    pub fn from_elf(name: &str, elf: &[u8]) -> Option<Task> {
        let tid = TidHandle::new()?;
        let pid = tid.0;
        // 创建进程地址空间
        let elf_info = build_elf_address_space(elf);
        if elf_info.is_err() {
            return None;
        }
        let elf_info = elf_info.unwrap();
        let address_space = elf_info.address_space;
        let k_stack = Stack::new(1)?;
        let k_stack_top = k_stack.top();
        let process = Task {
            tid,
            kernel_stack: k_stack,
            pid,
            inner: Mutex::new(TaskInner {
                name: name.to_string(),
                address_space: Arc::new(Mutex::new(address_space)),
                state: TaskState::Ready,
                parent: None,
                children: Vec::new(),
                fd_table: {
                    let mut fd_table = FdManager::new(MAX_FD_NUM);
                    fd_table.insert(STDIN.clone()).unwrap();
                    fd_table.insert(STDOUT.clone()).unwrap();
                    fd_table.insert(STDOUT.clone()).unwrap();
                    fd_table
                },
                context: Context::new(trap_return as usize, k_stack_top),
                fs_info: FsContext::empty(),
                statistical_data: StatisticalData::new(),
                exit_code: 0,
                heap: HeapInfo::new(elf_info.heap_bottom, elf_info.heap_bottom),
                mmap: MMapInfo::new(),
                signal_handlers: Arc::new(Mutex::new(SignalHandlers::new())),
                signal_receivers: Arc::new(Mutex::new(SignalReceivers::new())),
                set_child_tid: 0,
                clear_child_tid: 0,
                trap_cx_before_signal: None,
                signal_set_siginfo: false,
            }),
            send_sigchld_when_exit: false,
        };
        let phy_button = process.transfer_raw(elf_info.stack_top - USER_STACK_SIZE);
        let mut user_stack = UserStack::new(phy_button + USER_STACK_SIZE, elf_info.stack_top);
        user_stack.push(0).unwrap();
        let argc_ptr = user_stack.push(0).unwrap();

        let trap_frame = process.trap_frame();
        *trap_frame = TrapFrame::from_app_info(
            elf_info.entry,
            argc_ptr,
            kernel_satp(),
            process.kernel_stack.top(),
            user_trap_vector as usize,
        );
        Some(process)
    }
    /// fork a child
    pub fn t_clone(
        self: &Arc<Self>,
        flag: CloneFlags,
        stack: usize,
        _sig: SignalFlags,
        _ptid: usize,
        _tls: usize,
        _ctid: usize,
    ) -> Option<Arc<Task>> {
        assert_eq!(flag, CloneFlags::empty());
        assert_eq!(stack, 0);
        let tid = TidHandle::new()?;
        let mut inner = self.inner.lock();
        let address_space = build_clone_address_space(&mut inner.address_space.lock());
        let k_stack = Stack::new(USER_KERNEL_STACK_SIZE / FRAME_SIZE)?;
        let k_stack_top = k_stack.top();
        let pid = if flag.contains(CloneFlags::CLONE_THREAD) {
            self.pid
        } else {
            tid.0
        };
        let process = Task {
            tid,
            kernel_stack: k_stack,
            pid,
            inner: Mutex::new(TaskInner {
                name: inner.name.clone(),
                address_space: Arc::new(Mutex::new(address_space)),
                state: TaskState::Ready,
                parent: Some(Arc::downgrade(self)),
                children: Vec::new(),
                fd_table: inner.fd_table.clone(),
                context: Context::new(trap_return as usize, k_stack_top),
                fs_info: inner.fs_info.clone(),
                statistical_data: StatisticalData::new(),
                exit_code: 0,
                heap: inner.heap.clone(),
                mmap: inner.mmap.clone(),
                signal_handlers: inner.signal_handlers.clone(),
                signal_receivers: inner.signal_receivers.clone(),
                set_child_tid: 0,
                clear_child_tid: 0,
                trap_cx_before_signal: None,
                signal_set_siginfo: false,
            }),
            send_sigchld_when_exit: false,
        };
        let process = Arc::new(process);
        inner.children.push(process.clone());
        let trap_frame = process.trap_frame();
        trap_frame.update_kernel_sp(k_stack_top);

        if stack != 0 {
            // set the sp of the new process
            trap_frame.regs()[2] = stack;
        }

        Some(process)
    }

    #[no_mangle]
    pub fn exec(
        &self,
        name: &str,
        elf_data: &[u8],
        args: Vec<String>,
        env: Vec<String>,
    ) -> Result<(), isize> {
        let elf_info = build_elf_address_space(elf_data);
        if elf_info.is_err() {
            return Err(-1);
        }
        let elf_info = elf_info.unwrap();
        let mut inner = self.inner.lock();
        let address_space = elf_info.address_space;
        // reset the address space
        inner.address_space = Arc::new(Mutex::new(address_space));
        // reset the heap
        inner.heap = HeapInfo::new(elf_info.heap_bottom, elf_info.heap_bottom);
        // reset the mmap
        inner.mmap = MMapInfo::new();
        // set the name of the process
        inner.name = name.to_string();
        // reset time record
        inner.statistical_data.clear();
        // close file which contains FD_CLOEXEC flag
        // now we delete all fd
        // inner.fd_table = ;
        // reset signal handler
        inner.signal_handlers.lock().clear();
        inner.signal_receivers.lock().clear();

        // we need make sure the args and env size is less than 4KB
        let phy_button = inner.transfer_raw(elf_info.stack_top - FRAME_SIZE);
        let mut user_stack = UserStack::new(phy_button + FRAME_SIZE, elf_info.stack_top);
        // push env to the top of stack of the process
        // we have push '\0' into the env string,so we don't need to push it again
        let envv = env
            .iter()
            .map(|env| user_stack.push_str(env).unwrap())
            .collect::<Vec<usize>>();
        // push the args to the top of stack of the process
        // we have push '\0' into the arg string,so we don't need to push it again
        let argcv = args
            .iter()
            .map(|arg| user_stack.push_str(arg).unwrap())
            .collect::<Vec<usize>>();
        // push padding to the top of stack of the process
        user_stack.align_to(8).unwrap();
        let random_ptr = user_stack.push_bytes(&[0u8; 16]).unwrap();
        // padding
        user_stack.push_bytes(&[0u8; 8]).unwrap();
        // push aux
        let platform = user_stack.push_str("riscv").unwrap();
        let ex_path = user_stack.push_str(args[0].as_str()).unwrap();
        user_stack.push(0).unwrap();
        user_stack.push(platform).unwrap();
        user_stack.push(AT_PLATFORM).unwrap();
        user_stack.push(ex_path).unwrap();
        user_stack.push(AT_EXECFN).unwrap();
        user_stack.push(elf_info.ph_num).unwrap();
        user_stack.push(AT_PHNUM).unwrap();
        user_stack.push(FRAME_SIZE).unwrap();
        user_stack.push(AT_PAGESZ).unwrap();
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
        // psuh the env addr to the top of stack of the process
        envv.iter().for_each(|env| {
            user_stack.push(*env).unwrap();
        });
        user_stack.push(0).unwrap();
        // push the args addr to the top of stack of the process
        argcv.iter().skip(1).enumerate().for_each(|(_i, arg)| {
            user_stack.push(*arg).unwrap();
        });
        let record_first_arg = user_stack.push(argcv[0]).unwrap();
        // push the argc to the top of stack of the process
        let argc = args.len();
        let argc_ptr = user_stack.push(argc).unwrap();
        let user_sp = argc_ptr;
        warn!(
            "args:{:?}, env:{:?} argv: {:#x} user_sp: {:#x}",
            args, env, record_first_arg, user_sp
        );
        let (physical, _, _) = inner
            .address_space
            .lock()
            .query(VirtAddr::from(TRAP_CONTEXT_BASE))
            .unwrap();
        let trap_frame = TrapFrame::from_raw_ptr(physical.as_usize() as *mut TrapFrame);
        *trap_frame = TrapFrame::from_app_info(
            elf_info.entry,
            user_sp,
            kernel_satp(),
            self.kernel_stack.top(),
            user_trap_vector as usize,
        );
        Ok(())
    }
}
