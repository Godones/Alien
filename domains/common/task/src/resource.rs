use alloc::{collections::BTreeMap, string::String, sync::Arc, vec::Vec};
use core::fmt::{Debug, Formatter};

use basic::vm::frame::FrameTracker;
use config::{FRAME_SIZE, MAX_FD_NUM, MAX_THREAD_NUM, USER_STACK_SIZE};
use constants::{
    aux::{
        AT_BASE, AT_EGID, AT_ENTRY, AT_EUID, AT_EXECFN, AT_GID, AT_IGNORE, AT_PAGESZ, AT_PHDR,
        AT_PHENT, AT_PHNUM, AT_PLATFORM, AT_RANDOM, AT_SECURE, AT_UID,
    },
    AlienError, AlienResult,
};
use ksync::Mutex;
use memory_addr::VirtAddr;
use ptable::{VmIo, VmSpace};
use small_index::IndexAllocator;
use spin::Lazy;

use crate::{
    elf::{ELFInfo, VmmPageAllocator},
    vfs_shim::ShimFile,
};

pub static TID_MANAGER: Lazy<Mutex<IndexAllocator<MAX_THREAD_NUM>>> =
    Lazy::new(|| Mutex::new(IndexAllocator::new()));

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct TidHandle(pub usize);

impl TidHandle {
    /// 获取一个新的线程 tid (来自于 `TID_MANAGER` 分配)
    pub fn new() -> Option<Self> {
        let tid = TID_MANAGER.lock().allocate();
        if tid.is_err() {
            return None;
        }
        Some(Self(tid.unwrap()))
    }

    pub fn raw(&self) -> usize {
        self.0
    }
}

impl Drop for TidHandle {
    fn drop(&mut self) {
        TID_MANAGER.lock().deallocate(self.0).unwrap();
    }
}

#[derive(Clone)]
pub struct FdManager {
    index_map: IndexAllocator<MAX_FD_NUM>,
    fd_table: Vec<Option<Arc<ShimFile>>>,
}

impl Debug for FdManager {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FdManager")
            .field("index_map_size", &MAX_FD_NUM)
            .field("fd_table", &self.fd_table)
            .finish()
    }
}

impl FdManager {
    pub fn new() -> Self {
        let mut fd_table = Vec::with_capacity(MAX_FD_NUM);
        for _ in 0..MAX_FD_NUM {
            fd_table.push(None);
        }
        Self {
            index_map: IndexAllocator::new(),
            fd_table,
        }
    }
    pub fn get(&self, fd: usize) -> Option<Arc<ShimFile>> {
        if fd >= MAX_FD_NUM {
            return None;
        }
        self.fd_table[fd].clone()
    }

    pub fn insert(&mut self, file: Arc<ShimFile>) -> usize {
        let fd = self.index_map.allocate().unwrap();
        self.fd_table[fd] = Some(file);
        fd
    }
}

#[derive(Debug)]
pub struct KStack {
    frames: Option<FrameTracker>,
}

impl KStack {
    pub fn new(pages: usize) -> Self {
        let frames = FrameTracker::new(pages);
        Self {
            frames: Some(frames),
        }
    }

    pub fn top(&self) -> usize {
        self.frames.as_ref().unwrap().end_virt_addr().as_usize()
    }

    pub fn release(&mut self) {
        self.frames.take();
    }
}

#[derive(Debug, Clone)]
pub struct HeapInfo {
    /// 堆使用到的位置
    pub current: usize,
    /// 堆空间的起始位置
    pub start: usize,
    /// 堆空间的末尾位置
    pub end: usize,
}

impl HeapInfo {
    /// 新建一个 HeapInfo
    pub fn new(start: usize, end: usize) -> Self {
        HeapInfo {
            current: start,
            start,
            end,
        }
    }
}

#[derive(Debug)]
pub struct UserStack {
    virt_stack_top: VirtAddr,
    stack_size: usize,
    pos: VirtAddr,
    argv: Vec<String>,
    envp: Vec<String>,
    aux_vec: AuxVec,
    exec_path: String,
}

impl UserStack {
    pub fn new(
        virt_stack_top: VirtAddr,
        argv: Vec<String>,
        envp: Vec<String>,
        aux_vec: AuxVec,
        exec_path: String,
    ) -> Self {
        Self {
            virt_stack_top,
            stack_size: USER_STACK_SIZE,
            pos: virt_stack_top,
            argv,
            envp,
            aux_vec,
            exec_path,
        }
    }

    pub fn init(&mut self, vm_space: &mut VmSpace<VmmPageAllocator>) -> AlienResult<VirtAddr> {
        let envp_pointers = self.push_envp(vm_space)?;
        let argv_pointers = self.push_argv(vm_space)?;
        // push padding to the top of stack of the process
        self.align_to(8)?;
        self.push_aux_vec(vm_space)?;
        self.push_u64(0, vm_space)?;
        self.push_vec_u64(&envp_pointers, vm_space)?;
        self.push_u64(0, vm_space)?;
        self.push_vec_u64(&argv_pointers, vm_space)?;
        self.push_u64(self.argv.len() as _, vm_space)
    }
    pub fn top(&self) -> VirtAddr {
        self.pos
    }

    fn push_aux_vec(&mut self, vm_space: &mut VmSpace<VmmPageAllocator>) -> AlienResult<()> {
        let random_ptr = self.push_bytes(&[0u8; 16], vm_space)?;
        // padding
        self.push_bytes(&[0u8; 8], vm_space)?;
        // push platform and exec path
        let platform = self.push_str("riscv", vm_space)?;
        let ex_path = self.push_str(&self.exec_path.clone(), vm_space)?;
        self.aux_vec.set(AT_PLATFORM, platform.as_usize() as _)?;
        self.aux_vec.set(AT_EXECFN, ex_path.as_usize() as _)?;
        self.aux_vec.set(AT_RANDOM, random_ptr.as_usize() as _)?;

        self.push_u64(0, vm_space)?;
        for (key, val) in self.aux_vec.table().clone() {
            self.push_u64(val, vm_space)?;
            self.push_u64(key as _, vm_space)?;
        }
        Ok(())
    }

    fn push_argv(&mut self, vm_space: &mut VmSpace<VmmPageAllocator>) -> AlienResult<Vec<u64>> {
        let mut data_ptrs = Vec::with_capacity(self.argv.len());
        for data in self.argv.to_vec().iter().rev() {
            let addr = self.push_str(data, vm_space)?;
            data_ptrs.push(addr.as_usize() as _);
        }
        Ok(data_ptrs)
    }

    fn push_envp(&mut self, vm_space: &mut VmSpace<VmmPageAllocator>) -> AlienResult<Vec<u64>> {
        let mut data_ptrs = Vec::with_capacity(self.envp.len());
        for data in self.envp.to_vec().iter().rev() {
            let addr = self.push_str(data, vm_space)?;
            data_ptrs.push(addr.as_usize() as _);
        }
        Ok(data_ptrs)
    }

    fn push_vec_u64(
        &mut self,
        data: &Vec<u64>,
        vm_space: &mut VmSpace<VmmPageAllocator>,
    ) -> AlienResult<()> {
        for data in data.iter().rev() {
            self.push_u64(*data, vm_space)?;
        }
        Ok(())
    }

    pub fn push_u64(
        &mut self,
        data: u64,
        vm_space: &mut VmSpace<VmmPageAllocator>,
    ) -> AlienResult<VirtAddr> {
        if self.pos < self.virt_stack_top - self.stack_size {
            return Err(AlienError::ENOSPC);
        }
        self.pos -= 8;
        vm_space.write_val(self.pos, &data).unwrap();
        Ok(self.pos)
    }

    pub fn push_str(
        &mut self,
        data: &str,
        vm_space: &mut VmSpace<VmmPageAllocator>,
    ) -> AlienResult<VirtAddr> {
        self.push_bytes(data.as_bytes(), vm_space)
    }

    fn push_bytes(
        &mut self,
        data: &[u8],
        vm_space: &mut VmSpace<VmmPageAllocator>,
    ) -> AlienResult<VirtAddr> {
        let len = data.len();
        self.pos -= len;
        self.pos = self.pos.align_down(8usize);
        if self.pos < self.virt_stack_top - self.stack_size {
            return Err(AlienError::ENOSPC);
        }
        vm_space.write_bytes(self.pos, data).unwrap();
        Ok(self.pos)
    }

    fn align_to(&mut self, align: usize) -> AlienResult<VirtAddr> {
        let new_pos = self.pos.align_down(align);
        if new_pos < self.virt_stack_top - self.stack_size {
            return Err(AlienError::ENOSPC);
        }
        self.pos = new_pos;
        Ok(self.pos)
    }
}

#[derive(Clone, Default, Debug)]
pub struct AuxVec {
    table: BTreeMap<usize, u64>,
}

impl AuxVec {
    pub const fn new() -> AuxVec {
        AuxVec {
            table: BTreeMap::new(),
        }
    }
    pub fn from_elf_info(elfinfo: &ELFInfo) -> AlienResult<Self> {
        let mut auxvec = AuxVec::new();
        auxvec.set(AT_PHNUM, elfinfo.ph_num as _)?;
        auxvec.set(AT_PAGESZ, FRAME_SIZE as _)?;
        auxvec.set(AT_BASE, elfinfo.bias as _)?;
        auxvec.set(AT_ENTRY, elfinfo.entry.as_usize() as _)?;
        auxvec.set(AT_PHENT, elfinfo.ph_entry_size as _)?;
        auxvec.set(AT_PHDR, elfinfo.ph_drift as _)?;
        auxvec.set(AT_GID, 0)?;
        auxvec.set(AT_EGID, 0)?;
        auxvec.set(AT_UID, 0)?;
        auxvec.set(AT_EUID, 0)?;
        auxvec.set(AT_SECURE, 0)?;
        Ok(auxvec)
    }
}

impl AuxVec {
    pub fn set(&mut self, key: usize, val: u64) -> AlienResult<()> {
        if key == 0 || key == AT_IGNORE {
            return Err(AlienError::EINVAL);
        }
        self.table
            .entry(key)
            .and_modify(|val_mut| *val_mut = val)
            .or_insert(val);
        Ok(())
    }

    pub fn get(&self, key: usize) -> Option<u64> {
        self.table.get(&key).copied()
    }

    pub fn del(&mut self, key: usize) -> Option<u64> {
        self.table.remove(&key)
    }

    pub fn table(&self) -> &BTreeMap<usize, u64> {
        &self.table
    }
}
