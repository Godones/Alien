use core::ops::{Deref, DerefMut};

use basic::task::KStack;
use config::{FRAME_SIZE, USER_KERNEL_STACK_SIZE};
use task_meta::TaskMeta;

#[derive(Debug)]
pub struct TaskMetaExt {
    pub kstack: KStack,
    pub meta: TaskMeta,
}

impl TaskMetaExt {
    pub fn new(mut meta: TaskMeta) -> Self {
        let kstack = KStack::new(meta.tid, USER_KERNEL_STACK_SIZE / FRAME_SIZE);
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
