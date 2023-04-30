use alloc::string::String;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;

use lazy_static::lazy_static;
use page_table::{AddressSpace, PTableError, PTEFlags, vpn_f_c_range};
use page_table::VPN;
use rvfs::dentry::DirEntry;
use rvfs::file::File;
use rvfs::info::ProcessFsInfo;
use rvfs::mount::VfsMount;
use spin::{Mutex, MutexGuard};

use gmanager::MinimalManager;

use crate::config::{MAX_FD_NUM, MAX_PROCESS_NUM, TRAP_CONTEXT_BASE};
use crate::config::FRAME_SIZE;
use crate::error::AlienError;
use crate::fs::{STDIN, STDOUT};
use crate::memory::{build_elf_address_space, kernel_satp};
use crate::task::context::Context;
use crate::task::stack::Stack;
use crate::timer::read_timer;
use crate::trap::{trap_return, TrapFrame, user_trap_vector};

type FdManager = MinimalManager<Arc<File>>;

lazy_static! {
    /// 这里把MinimalManager复用为pid分配器，通常，MinimalManager会将数据插入到最小可用位置并返回位置，
    /// 但pid的分配并不需要实际存储信息，因此可以插入任意的数据，这里为了节省空间，将数据定义为u8
    pub static ref PID_MANAGER:Mutex<MinimalManager<u8>> = Mutex::new(MinimalManager::new(MAX_PROCESS_NUM));
}
#[derive(Debug)]
pub struct PidHandle(usize);

impl Drop for PidHandle {
    fn drop(&mut self) {
        PID_MANAGER.lock().remove(self.0).unwrap();
    }
}

#[derive(Debug)]
pub struct Process {
    pid: PidHandle,
    kernel_stack: Stack,
    inner: Mutex<ProcessInner>,
}

unsafe impl Send for Process {}

unsafe impl Sync for Process {}

#[derive(Debug)]
pub struct ProcessInner {
    pub address_space: AddressSpace,
    pub state: ProcessState,
    pub parent: Option<Weak<Process>>,
    pub children: Vec<Arc<Process>>,
    pub trap_frame: *mut TrapFrame,
    pub fd_table: FdManager,
    pub context: Context,
    pub fs_info: FsContext,
    pub statistical_data: StatisticalData,
    pub exit_code: i32,
    pub heap: HeapInfo,
}


#[derive(Debug, Clone)]
pub struct HeapInfo {
    pub start: usize,
    pub end: usize,
}

impl HeapInfo {
    pub fn new(start: usize, end: usize) -> Self {
        HeapInfo {
            start,
            end,
        }
    }

    #[allow(unused)]
    pub fn size(&self) -> usize {
        self.end - self.start
    }

    #[allow(unused)]
    pub fn contains(&self, addr: usize) -> bool {
        addr >= self.start && addr < self.end
    }

    pub fn increase(&mut self, size: usize) {
        self.end += size;
    }

    #[allow(unused)]
    pub fn set_start(&mut self, start: usize) {
        self.start = start;
    }

    pub fn set_end(&mut self, end: usize) {
        self.end = end;
    }

    #[allow(unused)]
    pub fn is_empty(&self) -> bool {
        self.start == self.end
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
pub enum ProcessState {
    Ready,
    Running,
    Sleeping,
    Zombie,
    Waiting,
}

impl Process {
    pub fn get_pid(&self) -> isize {
        self.pid.0 as isize
    }

    pub fn access_inner(&self) -> MutexGuard<ProcessInner> {
        self.inner.lock()
    }
    pub fn token(&self) -> usize {
        let inner = self.inner.lock();
        8usize << 60 | inner.address_space.root_ppn().unwrap().0
    }

    pub fn trap_frame(&self) -> &'static mut TrapFrame {
        let inner = self.inner.lock();
        TrapFrame::from_raw_ptr(inner.trap_frame)
    }

    pub fn update_state(&self, state: ProcessState) {
        let mut inner = self.inner.lock();
        inner.state = state;
    }

    pub fn state(&self) -> ProcessState {
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

    pub fn children(&self) -> Vec<Arc<Process>> {
        let inner = self.inner.lock();
        inner.children.clone()
    }
    pub fn remove_child(&self, index: usize) -> Arc<Process> {
        let mut inner = self.inner.lock();
        assert!(index < inner.children.len());
        inner.children.remove(index)
    }

    pub fn update_parent(&self, parent: Arc<Process>) {
        let mut inner = self.inner.lock();
        inner.parent = Some(Arc::downgrade(&parent));
    }

    pub fn insert_child(&self, child: Arc<Process>) {
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
}

impl ProcessInner {
    pub fn cwd(&self) -> FsContext {
        self.fs_info.clone()
    }
    pub fn transfer_raw(&self, ptr: usize) -> usize {
        self.address_space.virtual_to_physical(ptr).unwrap()
    }
    pub fn transfer_str(&self, ptr: *const u8) -> String {
        let mut res = String::new();
        let mut start = ptr as usize;
        loop {
            let physical = self.address_space.virtual_to_physical(start);
            if physical.is_none() {
                break;
            }
            let physical = physical.unwrap();
            let c = unsafe { &*(physical as *const u8) };
            if *c == 0 {
                break;
            }
            res.push(*c as char);
            start += 1;
        }
        res
    }
    pub fn transfer_raw_buffer(&self, ptr: *const u8, len: usize) -> Vec<&'static mut [u8]> {
        let address_space = &self.address_space;
        let mut start = ptr as usize;
        let end = start + len;
        let mut v = Vec::new();
        while start < end {
            let start_phy = address_space.virtual_to_physical(start).unwrap();
            // find the value >= start && value%FRAME_SIZE == 0
            let bound = (start_phy + FRAME_SIZE - 1) & !(FRAME_SIZE - 1);
            let len = if bound > end {
                end - start
            } else {
                bound - start
            };
            unsafe {
                let buf = core::slice::from_raw_parts_mut(start_phy as *mut u8, len);
                v.push(buf);
            }
            start = bound;
        }
        v
    }

    pub fn transfer_raw_ptr<T>(&self, ptr: *mut T) -> &'static mut T {
        let physical = self
            .address_space
            .virtual_to_physical(ptr as usize)
            .unwrap();
        unsafe { &mut *(physical as *mut T) }
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

    /// extend heap
    pub fn extend_heap(&mut self, addition: usize) -> Result<usize, AlienError> {
        // increase heap size
        let end = self.heap.end;
        // align addition to PAGE_SIZE
        let addition = (addition + FRAME_SIZE - 1) & !(FRAME_SIZE - 1);
        let vpn_range = vpn_f_c_range!(end, end + addition);
        vpn_range.for_each(|x| {
            self.address_space.push_with_vpn(x, PTEFlags::V | PTEFlags::W | PTEFlags::R | PTEFlags::U).map_err(|x| {
                match x {
                    PTableError::AllocError => panic!("alloc error,the memory is not enough"),
                    _ => {}
                }
            }).unwrap();
        });
        let new_end = end + addition;
        self.heap.end = new_end;
        Ok(end)
    }
}

impl Process {
    pub fn recycle(&self) {
        let mut inner = self.inner.lock();
        // delete child process
        inner.children.clear();
        // recycle page
        inner.address_space.recycle();
    }
    /// only call once
    pub fn from_elf(elf: &[u8]) -> Option<Process> {
        let pid = PID_MANAGER.lock().insert(0).unwrap();
        // 创建进程地址空间
        let elf_info = build_elf_address_space(elf);
        if elf_info.is_err() {
            return None;
        }
        let elf_info = elf_info.unwrap();
        // info!("elf_info: {:#x?}", elf_info);
        let address_space = elf_info.address_space;
        let physical = address_space.virtual_to_physical(TRAP_CONTEXT_BASE)?;
        let trap_frame = physical as *mut TrapFrame;
        let k_stack = Stack::new(1)?;
        let k_stack_top = k_stack.top();
        let process = Process {
            kernel_stack: k_stack,
            pid: PidHandle(pid),
            inner: Mutex::new(ProcessInner {
                address_space,
                state: ProcessState::Ready,
                parent: None,
                children: Vec::new(),
                trap_frame,
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
            }),
        };
        let trap_frame = process.trap_frame();
        *trap_frame = TrapFrame::from_app_info(
            elf_info.entry,
            elf_info.stack_top - 16,
            kernel_satp(),
            process.kernel_stack.top(),
            user_trap_vector as usize,
        );
        Some(process)
    }
    /// fork a child
    pub fn fork(self: &Arc<Self>) -> Option<Arc<Process>> {
        let pid = PID_MANAGER.lock().insert(0).unwrap();
        let mut inner = self.inner.lock();
        let address_space = AddressSpace::copy_from_other(&inner.address_space).ok()?;
        let physical = address_space
            .virtual_to_physical(TRAP_CONTEXT_BASE)
            .unwrap();
        let trap_frame = physical as *mut TrapFrame;
        let k_stack = Stack::new(1)?;
        let k_stack_top = k_stack.top();
        let process = Process {
            kernel_stack: k_stack,
            pid: PidHandle(pid),
            inner: Mutex::new(ProcessInner {
                address_space,
                state: ProcessState::Ready,
                parent: Some(Arc::downgrade(self)),
                children: Vec::new(),
                trap_frame,
                fd_table: inner.fd_table.clone(),
                context: Context::new(trap_return as usize, k_stack_top),
                fs_info: inner.fs_info.clone(),
                statistical_data: StatisticalData::new(),
                exit_code: 0,
                heap: inner.heap.clone(),
            }),
        };
        let process = Arc::new(process);
        inner.children.push(process.clone());
        let trap_frame = process.trap_frame();
        trap_frame.update_kernel_sp(k_stack_top);
        Some(process)
    }

    pub fn exec(&self, elf_data: &[u8], args: Vec<String>) -> Result<(), isize> {
        let elf_info = build_elf_address_space(elf_data);
        if elf_info.is_err() {
            return Err(-1);
        }
        let elf_info = elf_info.unwrap();
        let mut inner = self.inner.lock();
        let address_space = elf_info.address_space;
        let physical = address_space
            .virtual_to_physical(TRAP_CONTEXT_BASE)
            .unwrap();
        let trap_frame = physical as *mut TrapFrame;
        inner.address_space = address_space;
        inner.trap_frame = trap_frame;
        inner.heap = HeapInfo::new(elf_info.heap_bottom, elf_info.heap_bottom);

        // push the args to the top of stack of the process
        // we have push '\0' into the arg string,so we don't need to push it again
        let base = elf_info.stack_top - args.len() * core::mem::size_of::<usize>();
        let mut str_base = base;
        args.iter().enumerate().for_each(|(i, arg)| unsafe {
            let arg_addr = base + i * core::mem::size_of::<usize>();
            let arg_addr = inner.transfer_raw_ptr(arg_addr as *mut usize);
            str_base = str_base - arg.as_bytes().len();
            *arg_addr = str_base;
            let arg_str_addr = inner.transfer_raw(str_base);
            core::slice::from_raw_parts_mut(arg_str_addr as *mut u8, arg.as_bytes().len())
                .copy_from_slice(arg.as_bytes());
        });
        // align the user_sp to 8byte
        let user_sp = (str_base - 8) & !0x7;

        let trap_frame = TrapFrame::from_raw_ptr(trap_frame);
        *trap_frame = TrapFrame::from_app_info(
            elf_info.entry,
            user_sp,
            kernel_satp(),
            self.kernel_stack.top(),
            user_trap_vector as usize,
        );
        trap_frame.regs()[10] = args.len() as usize;
        trap_frame.regs()[11] = base;
        Ok(())
    }
}
