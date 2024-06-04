use core::ops::{Deref, DerefMut};

use config::{FRAME_SIZE, KTHREAD_STACK_SIZE, USER_KERNEL_STACK_SIZE};
use mem::VirtAddr;
use rref::RRef;
use task_meta::{TaskBasicInfo, TaskMeta, TaskSchedulingInfo};

use crate::task::continuation::ContinuationManager;

#[derive(Debug)]
pub struct KStack {
    top: VirtAddr,
    tid: usize,
    pages: usize,
}

impl KStack {
    pub fn new(tid: usize, pages: usize) -> Self {
        let top = mem::map_kstack_for_task(tid, pages).expect("map kstack failed");
        Self {
            top: VirtAddr::from(top),
            tid,
            pages,
        }
    }

    pub fn top(&self) -> VirtAddr {
        self.top
    }
}

impl Drop for KStack {
    fn drop(&mut self) {
        mem::unmap_kstack_for_task(self.tid, self.pages).expect("unmap kstack failed");
    }
}

#[derive(Debug)]
pub struct TaskMetaExt {
    pub kstack: KStack,
    pub basic_info: TaskBasicInfo,
    pub scheduling_info: Option<RRef<TaskSchedulingInfo>>,
    pub continuation: ContinuationManager,
}

impl TaskMetaExt {
    pub fn new(meta: TaskMeta, is_kthread: bool) -> Self {
        let mut basic_info = meta.task_basic_info;
        let scheduling_info = meta.scheduling_info;
        let size = if is_kthread {
            KTHREAD_STACK_SIZE
        } else {
            USER_KERNEL_STACK_SIZE
        };
        let kstack = KStack::new(basic_info.tid, size / FRAME_SIZE);
        basic_info.context.set_sp(kstack.top().as_usize());
        Self {
            kstack,
            basic_info,
            scheduling_info: Some(RRef::new(scheduling_info)),
            continuation: ContinuationManager::new(),
        }
    }
    pub fn take_scheduling_info(&mut self) -> RRef<TaskSchedulingInfo> {
        self.scheduling_info
            .take()
            .expect("scheduling_info is None")
    }
    pub fn set_sched_info(&mut self, sched_info: RRef<TaskSchedulingInfo>) {
        self.scheduling_info = Some(sched_info);
    }
}

impl Deref for TaskMetaExt {
    type Target = TaskBasicInfo;

    fn deref(&self) -> &Self::Target {
        &self.basic_info
    }
}

impl DerefMut for TaskMetaExt {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.basic_info
    }
}
