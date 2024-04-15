use alloc::{collections::BTreeMap, sync::Arc};

use basic::sync::Mutex;
use rref::RRef;

use crate::{scheduler_domain, task::Task};

pub fn current_task() -> Option<Arc<Task>> {
    let tid = scheduler_domain!().current_tid().unwrap()?;
    let task = GLOBAL_TASK_MANAGER
        .lock()
        .get(&tid)
        .map(|task| Arc::clone(task));
    task
}

static GLOBAL_TASK_MANAGER: Mutex<BTreeMap<usize, Arc<Task>>> = Mutex::new(BTreeMap::new());

pub fn add_task(task: Arc<Task>) {
    let tid = task.tid();
    let task_meta = task.create_task_meta();
    GLOBAL_TASK_MANAGER.lock().insert(tid, task);
    scheduler_domain!()
        .add_one_task(RRef::new(task_meta))
        .unwrap()
}

pub fn remove_task(tid: usize) {
    GLOBAL_TASK_MANAGER.lock().remove(&tid);
    // scheduler_domain!().remove_one_task(tid).unwrap()
}
