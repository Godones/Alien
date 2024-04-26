use core::ops::{Deref, DerefMut};

use basic::vm::frame::FrameTracker;
use config::{FRAME_SIZE, USER_KERNEL_STACK_SIZE};
use memory_addr::VirtAddr;
use task_meta::TaskMeta;

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

    pub fn top(&self) -> VirtAddr {
        self.frames.as_ref().unwrap().end_virt_addr()
    }

    pub fn release(&mut self) {
        self.frames.take();
    }
}

#[derive(Debug)]
pub struct TaskMetaExt {
    pub kstack: KStack,
    pub meta: TaskMeta,
}

impl TaskMetaExt {
    pub fn new(mut meta: TaskMeta) -> Self {
        let kstack = KStack::new(USER_KERNEL_STACK_SIZE / FRAME_SIZE);
        meta.context.set_sp(kstack.top().as_usize());
        Self { kstack, meta }
    }
}

impl Deref for TaskMetaExt {
    type Target = TaskMeta;

    fn deref(&self) -> &Self::Target {
        &self.meta
    }
}

impl DerefMut for TaskMetaExt {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.meta
    }
}
