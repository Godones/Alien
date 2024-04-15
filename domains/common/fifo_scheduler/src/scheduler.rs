use alloc::{
    collections::{BTreeMap, VecDeque},
    sync::Arc,
};

use basic::sync::Mutex;
use task_meta::{TaskMeta, TaskStatus};

use crate::processor::{switch_to_cpu, take_current_task};

#[derive(Debug)]
pub struct FiFoScheduler {
    tasks: Mutex<VecDeque<Arc<Mutex<TaskMeta>>>>,
}

impl FiFoScheduler {
    pub const fn new() -> Self {
        Self {
            tasks: Mutex::new(VecDeque::new()),
        }
    }
}

static GLOBAL_SCHEDULER: FiFoScheduler = FiFoScheduler::new();

pub fn add_task(task_meta: Arc<Mutex<TaskMeta>>) {
    // log::info!("<add_task>: {:?}", task_meta.lock().tid());
    GLOBAL_SCHEDULER.tasks.lock().push_back(task_meta);
}

pub fn fetch_task() -> Option<Arc<Mutex<TaskMeta>>> {
    GLOBAL_SCHEDULER.tasks.lock().pop_front()
}
type Tid = usize;
static TASK_WAIT_QUEUE: Mutex<BTreeMap<Tid, Arc<Mutex<TaskMeta>>>> = Mutex::new(BTreeMap::new());

pub fn current_to_wait() {
    let task = take_current_task().unwrap();
    task.lock().set_status(TaskStatus::Waiting);
    let tid = task.lock().tid();
    TASK_WAIT_QUEUE.lock().insert(tid, task.clone());
    switch_to_cpu(task);
}

pub fn wake_up_wait_task(tid: Tid) {
    let task = TASK_WAIT_QUEUE.lock().remove(&tid);
    if let Some(task) = task {
        // put the task into the global task queue
        task.lock().set_status(TaskStatus::Ready);
        add_task(task);
    }
}

pub fn do_suspend() {
    let task = take_current_task().unwrap();
    // task.access_inner().update_timer();
    // check_task_timer_expired();
    task.lock().set_status(TaskStatus::Ready);
    switch_to_cpu(task);
}
