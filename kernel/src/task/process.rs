use crate::fs::{STDIN, STDOUT};
use alloc::string::String;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use gmanager::MinimalManager;
use lazy_static::lazy_static;
use page_table::AddressSpace;

use crate::config::FRAME_SIZE;
use crate::config::{MAX_FD_NUM, MAX_PROCESS_NUM, TRAP_CONTEXT_BASE};
use crate::memory::{build_elf_address_space, kernel_satp};
use crate::task::context::Context;
use crate::task::stack::Stack;
use crate::trap::{trap_return, user_trap_vector, TrapFrame};
use rvfs::dentry::DirEntry;
use rvfs::file::File;
use rvfs::info::ProcessFsInfo;
use rvfs::mount::VfsMount;
use spin::{Mutex, MutexGuard};
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
    pub exit_code: i32,
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

    pub fn transfer_raw(&self, ptr: usize) -> usize {
        let inner = self.inner.lock();
        inner.address_space.virtual_to_physical(ptr).unwrap()
    }

    // TODO 处理跨页问题
    pub fn transfer_raw_ptr<T>(&self, ptr: *mut T) -> &'static mut T {
        let inner = self.inner.lock();
        let physical = inner
            .address_space
            .virtual_to_physical(ptr as usize)
            .unwrap();
        unsafe { &mut *(physical as *mut T) }
    }
    // TODO:处理效率低的问题
    pub fn transfer_str(&self, ptr: *const u8) -> String {
        let inner = self.inner.lock();
        let mut res = String::new();
        let mut start = ptr as usize;
        loop {
            let physical = inner.address_space.virtual_to_physical(start).unwrap();
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
        let inner = self.inner.lock();
        let address_space = &inner.address_space;
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
                exit_code: 0,
            }),
        };
        let trap_frame = process.trap_frame();
        *trap_frame = TrapFrame::from_app_info(
            elf_info.entry,
            elf_info.stack_top,
            kernel_satp(),
            process.kernel_stack.top(),
            user_trap_vector as usize,
        );
        Some(process)
    }
    // fork a child
    pub fn fork(self: &Arc<Self>) -> Option<Arc<Process>> {
        let pid = PID_MANAGER.lock().insert(0).unwrap();
        let mut inner = self.inner.lock();
        let address_space = AddressSpace::copy_from_other(&inner.address_space);
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
                exit_code: 0,
            }),
        };
        let process = Arc::new(process);
        inner.children.push(process.clone());
        let trap_frame = process.trap_frame();
        trap_frame.update_kernel_sp(k_stack_top);
        Some(process)
    }
    pub fn exec(&self, elf_data: &[u8]) -> Result<(), isize> {
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
        let trap_frame = TrapFrame::from_raw_ptr(trap_frame);
        *trap_frame = TrapFrame::from_app_info(
            elf_info.entry,
            elf_info.stack_top,
            kernel_satp(),
            self.kernel_stack.top(),
            user_trap_vector as usize,
        );
        Ok(())
    }
}
