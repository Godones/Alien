use crate::vfs_shim::ShimFile;
use alloc::sync::Arc;
use alloc::vec::Vec;
use basic::frame::FrameTracker;
use config::{FRAME_SIZE, MAX_FD_NUM, MAX_THREAD_NUM};
use core::fmt::{Debug, Formatter};
use ksync::Mutex;
use small_index::IndexAllocator;
use spin::Lazy;

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
        let mut fd_table = Vec::new();
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
        self.frames.as_ref().unwrap().end()
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
    virt_stack_top: usize,
    stack_top: usize,
    stack_bottom: usize,
}

impl UserStack {
    pub fn new(phy_stack_top: usize, virt_stack_top: usize) -> Self {
        Self {
            virt_stack_top,
            stack_top: phy_stack_top,
            stack_bottom: phy_stack_top - FRAME_SIZE,
        }
    }
    pub fn push(&mut self, data: usize) -> Result<usize, &'static str> {
        if self.stack_top - 8 < self.stack_bottom {
            return Err("Stack Overflow");
        }
        unsafe {
            self.stack_top -= 8;
            *(self.stack_top as *mut usize) = data;
        }
        trace!(
            "stack top: {:#x}, data:{:#x?}",
            self.virt_stack_top - (FRAME_SIZE - (self.stack_top - self.stack_bottom)),
            data
        );
        Ok(self.virt_stack_top - (FRAME_SIZE - (self.stack_top - self.stack_bottom)))
    }

    pub fn push_str(&mut self, data: &str) -> Result<usize, &'static str> {
        self.push_bytes(data.as_bytes())
    }

    pub fn push_bytes(&mut self, data: &[u8]) -> Result<usize, &'static str> {
        let len = data.len();
        // align 8
        let start = self.stack_top - len;
        let start = start & !7;
        if start < self.stack_bottom {
            return Err("Stack Overflow");
        }
        unsafe {
            self.stack_top = start;
            let ptr = self.stack_top as *mut u8;
            ptr.copy_from_nonoverlapping(data.as_ptr(), len);
        }
        trace!(
            "stack top: {:#x}",
            self.virt_stack_top - (FRAME_SIZE - (self.stack_top - self.stack_bottom))
        );
        Ok(self.virt_stack_top - (FRAME_SIZE - (self.stack_top - self.stack_bottom)))
    }

    pub fn align_to(&mut self, align: usize) -> Result<usize, &'static str> {
        let start = self.stack_top & !(align - 1);
        if start < self.stack_bottom {
            return Err("Stack Overflow");
        }
        self.stack_top = start;
        Ok(self.virt_stack_top - (FRAME_SIZE - (self.stack_top - self.stack_bottom)))
    }
}
