use alloc::{collections::BTreeMap, sync::Arc};

use basic::sync::Mutex;
use spin::Lazy;

use crate::{
    processor::{add_task, take_current_task},
    scheduler::schedule_now,
    task::{Task, TaskStatus},
};

type Tid = usize;
static TASK_WAIT_QUEUE: Lazy<Mutex<BTreeMap<Tid, Arc<Task>>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));

/// Put the current task into the wait queue and schedule a new task to run.
pub fn current_to_wait() {
    let task = take_current_task().unwrap();
    task.update_state(TaskStatus::Waiting);
    let tid = task.tid.raw();
    TASK_WAIT_QUEUE.lock().insert(tid, task.clone());
    schedule_now(task);
}

pub fn wake_up_wait_task(tid: Tid) {
    let task = TASK_WAIT_QUEUE.lock().remove(&tid);
    if let Some(task) = task {
        // put the task into the global task queue
        task.update_state(TaskStatus::Ready);
        add_task(task);
    }
}
