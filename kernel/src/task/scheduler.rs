use alloc::{collections::BTreeMap, sync::Arc};

use basic::sync::Mutex;
use interface::SchedulerDomain;
use spin::Once;
use task_meta::TaskStatus;

use super::{processor::schedule, resource::TaskMetaExt};
use crate::task::processor::current_task;
type Tid = usize;
static TASK_MAP: Mutex<BTreeMap<Tid, Arc<Mutex<TaskMetaExt>>>> = Mutex::new(BTreeMap::new());
pub(super) static GLOBAL_SCHEDULER: Once<Arc<dyn SchedulerDomain>> = Once::new();

#[macro_export]
macro_rules! global_scheduler {
    () => {
        GLOBAL_SCHEDULER.get().unwrap()
    };
}

pub fn set_scheduler(scheduler: Arc<dyn SchedulerDomain>) {
    GLOBAL_SCHEDULER.call_once(|| scheduler);
}

pub fn add_task(task_meta: Arc<Mutex<TaskMetaExt>>) {
    // log::info!("<add_task>: {:?}", task_meta.lock().tid());
    let scheduling_info = task_meta.lock().take_scheduling_info();
    TASK_MAP.lock().insert(scheduling_info.tid, task_meta);
    global_scheduler!().add_task(scheduling_info).unwrap();
}

pub fn fetch_task() -> Option<Arc<Mutex<TaskMetaExt>>> {
    let scheduling_info = global_scheduler!().fetch_task().unwrap();
    if let Some(scheduling_info) = scheduling_info {
        let task = TASK_MAP.lock().remove(&scheduling_info.tid).unwrap();
        task.lock().set_sched_info(scheduling_info);
        return Some(task);
    }
    None
}
pub static TASK_WAIT_QUEUE: Mutex<BTreeMap<Tid, Arc<Mutex<TaskMetaExt>>>> =
    Mutex::new(BTreeMap::new());
pub static TASK_EXIT_QUEUE: Mutex<BTreeMap<Tid, Arc<Mutex<TaskMetaExt>>>> =
    Mutex::new(BTreeMap::new());

pub fn wait_now() {
    let task = current_task().unwrap();
    task.lock().set_status(TaskStatus::Waiting);
    let tid = task.lock().tid();
    TASK_WAIT_QUEUE.lock().insert(tid, task);
    schedule();
}

pub fn wake_up_wait_task(tid: Tid) {
    let task = TASK_WAIT_QUEUE.lock().remove(&tid);
    if let Some(task) = task {
        // put the task into the global task queue
        task.lock().set_status(TaskStatus::Ready);
        add_task(task);
    }
}

pub fn yield_now() {
    let task = current_task().unwrap();
    task.lock().set_status(TaskStatus::Ready);
    schedule();
}

pub fn exit_now() {
    let task = current_task().unwrap();
    let tid = task.lock().tid();
    task.lock().set_status(TaskStatus::Terminated);
    TASK_EXIT_QUEUE.lock().insert(tid, task);
    schedule();
}

pub fn remove_task(tid: Tid) {
    TASK_EXIT_QUEUE.lock().remove(&tid);
}
