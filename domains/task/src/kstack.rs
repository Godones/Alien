use libsyscall::{alloc_pages, FrameTracker};
#[derive(Debug)]
pub struct KStack {
    frames: Option<FrameTracker>,
}

impl KStack {
    pub fn new(pages: usize) -> Self {
        let frames = alloc_pages(pages);
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
