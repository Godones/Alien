use crate::processor::{add_task, take_current_task};
use crate::scheduler::schedule_now;
use crate::task::{Task, TaskState};
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use ksync::Mutex;
use spin::Lazy;

type Tid = usize;
static TASK_WAIT_QUEUE: Lazy<Mutex<BTreeMap<Tid, Arc<Task>>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));

/// Put the current task into the wait queue and schedule a new task to run.
pub fn current_to_wait() {
    let task = take_current_task().unwrap();
    task.update_state(TaskState::Waiting);
    let tid = task.tid.raw();
    TASK_WAIT_QUEUE.lock().insert(tid, task.clone());
    schedule_now(task);
}

pub fn wake_up_wait_task(tid: Tid) {
    let task = TASK_WAIT_QUEUE.lock().remove(&tid);
    if let Some(task) = task {
        // put the task into the global task queue
        task.update_state(TaskState::Ready);
        add_task(task);
    }
}
