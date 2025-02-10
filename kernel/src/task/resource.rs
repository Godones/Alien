use core::ops::{Deref, DerefMut};

use config::{FRAME_SIZE, KTHREAD_STACK_SIZE, USER_KERNEL_STACK_SIZE};
use mem::{FrameTracker, VirtAddr};
use shared_heap::DBox;
use task_meta::{TaskBasicInfo, TaskMeta, TaskSchedulingInfo};

#[derive(Debug)]
#[allow(unused)]
pub struct KStack {
    top: VirtAddr,
    tid: usize,
    pages: usize,
    frame_tracker: FrameTracker,
}

impl KStack {
    pub fn new(tid: usize, pages: usize) -> Self {
        // let top = mem::map_kstack_for_task(tid, pages).expect("map kstack failed");
        assert_eq!(pages.next_power_of_two(), pages);
        let frame = mem::alloc_frame_trackers(pages);
        let top = frame.start() + frame.len();
        Self {
            top: VirtAddr::from(top),
            tid,
            pages,
            frame_tracker: frame,
        }
    }

    pub fn top(&self) -> VirtAddr {
        self.top
    }
}

impl Drop for KStack {
    fn drop(&mut self) {
        // mem::unmap_kstack_for_task(self.tid, self.pages).expect("unmap kstack failed");
    }
}

#[derive(Debug)]
pub struct TaskMetaExt {
    pub kstack: KStack,
    pub basic_info: TaskBasicInfo,
    pub scheduling_info: Option<DBox<TaskSchedulingInfo>>,
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
            scheduling_info: Some(DBox::new(scheduling_info)),
        }
    }
    pub fn take_scheduling_info(&mut self) -> DBox<TaskSchedulingInfo> {
        self.scheduling_info
            .take()
            .expect("scheduling_info is None")
    }
    pub fn set_sched_info(&mut self, sched_info: DBox<TaskSchedulingInfo>) {
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
