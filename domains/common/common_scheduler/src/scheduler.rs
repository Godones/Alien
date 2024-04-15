use alloc::{boxed::Box, collections::BTreeMap, sync::Arc};

use basic::sync::Mutex;
use task_meta::{TaskMeta, TaskStatus};

use crate::processor::{switch_to_cpu, take_current_task};

pub trait Scheduler: Send + Sync {
    fn add_task(&mut self, task_meta: Arc<Mutex<TaskMeta>>);
    fn fetch_task(&mut self) -> Option<Arc<Mutex<TaskMeta>>>;
    fn name(&self) -> &'static str;
}

pub struct GlobalScheduler {
    scheduler: Option<Box<dyn Scheduler>>,
}

impl GlobalScheduler {
    pub fn set_scheduler(&mut self, scheduler: Box<dyn Scheduler>) {
        self.scheduler = Some(scheduler);
    }
}

impl Scheduler for GlobalScheduler {
    fn add_task(&mut self, task_meta: Arc<Mutex<TaskMeta>>) {
        self.scheduler.as_mut().unwrap().add_task(task_meta);
    }

    fn fetch_task(&mut self) -> Option<Arc<Mutex<TaskMeta>>> {
        self.scheduler.as_mut().unwrap().fetch_task()
    }
    fn name(&self) -> &'static str {
        self.scheduler.as_ref().unwrap().name()
    }
}

static GLOBAL_SCHEDULER: Mutex<GlobalScheduler> = Mutex::new(GlobalScheduler { scheduler: None });

pub fn set_scheduler(scheduler: Box<dyn Scheduler>) {
    GLOBAL_SCHEDULER.lock().set_scheduler(scheduler);
}

pub fn add_task(task_meta: Arc<Mutex<TaskMeta>>) {
    // log::info!("<add_task>: {:?}", task_meta.lock().tid());
    GLOBAL_SCHEDULER.lock().add_task(task_meta);
}

pub fn fetch_task() -> Option<Arc<Mutex<TaskMeta>>> {
    GLOBAL_SCHEDULER.lock().fetch_task()
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
