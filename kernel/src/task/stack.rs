use alloc::vec::Vec;

use kernel_sync::Mutex;

use crate::config::FRAME_BITS;
use crate::memory::{frame_alloc_contiguous, FrameTracker};

#[derive(Debug)]
pub struct Stack {
    frames: Mutex<Vec<FrameTracker>>,
}

impl Stack {
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
    /// get the stack top
    pub fn top(&self) -> usize {
        let first = self.frames.lock().last().map(|frame| frame.end());
        first.unwrap()
    }

    pub fn release(&self) {
        let mut frames = self.frames.lock();
        *frames = Vec::new();
    }
}
