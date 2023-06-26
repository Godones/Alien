use alloc::string::String;
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

use crate::config::{FRAME_BITS, MAX_FD_NUM, MAX_PROCESS_NUM, TRAP_CONTEXT_BASE};
use crate::config::FRAME_SIZE;
use crate::error::{AlienError, AlienResult};
use crate::fs::{STDIN, STDOUT};
use crate::memory::{
    build_clone_address_space, build_elf_address_space, FRAME_REF_MANAGER, kernel_satp, MapFlags,
    MMapInfo, MMapRegion, PageAllocator, ProtFlags,
};
use crate::task::context::Context;
use crate::task::cpu::{CloneFlags, SignalFlags};
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
    pub address_space: Sv39PageTable<PageAllocator>,
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
    pub mmap: MMapInfo,
}

#[derive(Debug, Clone)]
pub struct HeapInfo {
    pub current: usize,
    pub start: usize,
    pub end: usize,
}

impl HeapInfo {
    pub fn new(start: usize, end: usize) -> Self {
        HeapInfo {
            current: start,
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
        let paddr = inner.address_space.root_paddr();
        (8usize << 60) | (paddr.as_usize() >> 12)
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
    pub fn transfer_buffer<T>(&self, ptr: *const T, len: usize) -> Vec<&'static mut [T]> {
        self.access_inner().transfer_buffer(ptr, len)
    }
}

impl ProcessInner {
    pub fn cwd(&self) -> FsContext {
        self.fs_info.clone()
    }
    pub fn transfer_raw(&self, ptr: usize) -> usize {
        let (phy, ..) = self.address_space.query(VirtAddr::from(ptr)).unwrap();
        phy.as_usize()
    }
    pub fn transfer_str(&self, ptr: *const u8) -> String {
        let mut res = String::new();
        let mut start = ptr as usize;
        loop {
            let physical = self.address_space.query(VirtAddr::from(start));
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
        let address_space = &self.address_space;
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
        let address_space = &self.address_space;
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
            .map_region_no_target(
                VirtAddr::from(end),
                addition,
                "RWUAD".into(), // no V flag
                true,
                true)
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
        let (_, flags, _) = self.address_space.query(VirtAddr::from(addr)).unwrap();
        assert!(!flags.contains(MappingFlags::V));
        self.address_space
            .validate(VirtAddr::from(addr), map_flags)
            .unwrap();
        let (phy, _, size) = self.address_space.query(VirtAddr::from(addr)).unwrap();
        let buf =
            unsafe { core::slice::from_raw_parts_mut(phy.as_usize() as *mut u8, size.into()) };
        let file = &region.fd;

        let read_offset = region.offset + (addr - region.start);
        Ok((file.clone(), buf, read_offset as u64))
    }

    fn invalid_page_solver(&mut self, addr: usize) -> AlienResult<Option<(Arc<File>, &'static mut [u8], u64)>> {
        let is_mmap = self.mmap.get_region(addr);
        let is_heap = self.heap.contains(addr);
        if is_mmap.is_none() && !is_heap {
            return Err(AlienError::Other);
        }
        if is_heap {
            trace!("invalid page fault in heap");
            let map_flags = "RWUVAD".into();
            self.address_space
                .validate(VirtAddr::from(addr), map_flags)
                .unwrap();
        } else {
            let region = is_mmap.unwrap();
            assert_eq!(addr % FRAME_SIZE, 0);
            // update page table
            let mut map_flags = region.prot.into();
            map_flags |= "V".into();
            self.address_space
                .validate(VirtAddr::from(addr), map_flags)
                .unwrap();
            let (phy, _, size) = self.address_space.query(VirtAddr::from(addr)).unwrap();
            let buf =
                unsafe { core::slice::from_raw_parts_mut(phy.as_usize() as *mut u8, size.into()) };
            let file = &region.fd;
            let read_offset = region.offset + (addr - region.start);
            return Ok(Some((file.clone(), buf, read_offset as u64)));
        }
        Ok(None)
    }

    pub fn do_store_page_fault(&mut self, addr: usize) -> AlienResult<Option<(Arc<File>, &'static mut [u8], u64)>> {
        trace!("do store page fault:{:#x}",addr);
        let addr = align_down_4k(addr);
        let (phy, flags, page_size) = self.address_space.query(VirtAddr::from(addr)).unwrap();
        if !flags.contains(MappingFlags::V) {
            return self.invalid_page_solver(addr);
        }
        assert!(flags.contains(MappingFlags::RSD), "flags:{:#x}", flags);
        // decrease the reference count
        let mut flags = flags | "W".into();
        flags -= MappingFlags::RSD;
        let new_phy = self
            .address_space
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

impl Process {
    pub fn recycle(&self) {
        let mut inner = self.inner.lock();
        // delete child process
        inner.children.clear();
        // recycle page
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
        let address_space = elf_info.address_space;
        let (physical, _, _) = address_space
            .query(VirtAddr::from(TRAP_CONTEXT_BASE))
            .unwrap();
        let trap_frame = physical.as_usize() as *mut TrapFrame;
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
                mmap: MMapInfo::new(),
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
    pub fn t_clone(
        self: &Arc<Self>,
        flag: CloneFlags,
        stack: usize,
        _sig: SignalFlags,
        _ptid: usize,
        _tls: usize,
        _ctid: usize,
    ) -> Option<Arc<Process>> {
        assert_eq!(flag, CloneFlags::empty());
        let pid = PID_MANAGER.lock().insert(0).unwrap();
        let mut inner = self.inner.lock();
        let address_space = build_clone_address_space(&mut inner.address_space);
        let (physical, _, _) = address_space
            .query(VirtAddr::from(TRAP_CONTEXT_BASE))
            .unwrap();
        let trap_frame = physical.as_usize() as *mut TrapFrame;
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
                mmap: inner.mmap.clone(),
            }),
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

    pub fn exec(&self, elf_data: &[u8], args: Vec<String>) -> Result<(), isize> {
        let elf_info = build_elf_address_space(elf_data);
        if elf_info.is_err() {
            return Err(-1);
        }
        let elf_info = elf_info.unwrap();
        let mut inner = self.inner.lock();
        let address_space = elf_info.address_space;
        let (physical, _, _) = address_space
            .query(VirtAddr::from(TRAP_CONTEXT_BASE))
            .unwrap();
        let trap_frame = physical.as_usize() as *mut TrapFrame;
        inner.address_space = address_space;
        inner.trap_frame = trap_frame;
        inner.heap = HeapInfo::new(elf_info.heap_bottom, elf_info.heap_bottom);
        inner.mmap = MMapInfo::new();

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
