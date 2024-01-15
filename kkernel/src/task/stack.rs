//! 进程内核栈空间

use config::{ FRAME_SIZE};
use mem::{alloc_frames, free_frames};

/// 记录进程内核栈空间
#[derive(Debug)]
pub struct Stack {
    /// 栈帧
    start_ptr: usize,
    pages: usize,
}

impl Stack {
    /// 通过帧的个数创建一块新的 内核栈
    pub fn new(pages: usize) -> Option<Stack> {
      let frames = alloc_frames(pages);
        Some(Stack {
            start_ptr:frames as usize,
            pages
        })
    }

    /// 获取帧顶指针
    pub fn top(&self) -> usize {
        self.start_ptr + self.pages * FRAME_SIZE
    }

    /// 回收内核栈空间。
    pub fn release(&self) {
        free_frames(self.start_ptr as _, self.pages)
    }
}
