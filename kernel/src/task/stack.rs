//! 进程内核栈空间
use alloc::vec::Vec;

use crate::ksync::Mutex;

use crate::config::FRAME_BITS;
use crate::memory::{frame_alloc_contiguous, FrameTracker};

/// 记录进程内核栈空间
#[derive(Debug)]
pub struct Stack {
    /// 栈帧
    frames: Mutex<Vec<FrameTracker>>,
}

impl Stack {
    /// 通过帧的个数创建一块新的 内核栈
    pub fn new(pages: usize) -> Option<Stack> {
        let frames = frame_alloc_contiguous(pages) as usize >> FRAME_BITS;
        let frames = (0..pages)
            .into_iter()
            .map(|i| FrameTracker::new(i + frames))
            .collect::<Vec<FrameTracker>>();
        Some(Stack {
            frames: Mutex::new(frames),
        })
    }

    /// 获取帧顶指针
    pub fn top(&self) -> usize {
        let first = self.frames.lock().last().map(|frame| frame.end());
        first.unwrap()
    }

    /// 回收内核栈空间。
    pub fn release(&self) {
        let mut frames = self.frames.lock();
        *frames = Vec::new();
    }
}
