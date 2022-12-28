use crate::memory::{frames_alloc, FrameTracker};
use alloc::vec::Vec;

/// stack 拥有物理页帧的所有权
pub struct Stack {
    frames: Vec<FrameTracker>,
}

impl Stack {
    pub fn new(size: usize) -> Option<Stack> {
        let frames = frames_alloc(size);
        frames.map(|frames| Self { frames })
    }
    /// get the stack top
    pub fn top(&self) -> usize {
        let first = self.frames.last().map(|frame| frame.end());
        first.unwrap()
    }
}
