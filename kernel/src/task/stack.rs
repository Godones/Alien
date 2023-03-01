use crate::memory::{frames_alloc, FrameTracker};
use alloc::vec::Vec;

#[derive(Debug)]
pub struct Stack {
    frames: Vec<FrameTracker>,
}

impl Stack {
    pub fn new(pages: usize) -> Option<Stack> {
        let frames = frames_alloc(pages);
        match frames {
            Some(v) => Some(Stack { frames: v }),
            _ => None,
        }
    }
    /// get the stack top
    pub fn top(&self) -> usize {
        let first = self.frames.last().map(|frame| frame.end());
        first.unwrap()
    }
}
