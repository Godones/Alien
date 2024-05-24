use alloc::sync::Arc;
use core::cell::RefCell;

use basic::{arch::CpuLocal, sync::Mutex};
use task_meta::{TaskContext, TaskStatus};

use crate::task::{
    resource::TaskMetaExt,
    scheduler::{add_task, fetch_task},
};

#[derive(Debug, Clone)]
pub struct CPU {
    pub(crate) task: RefCell<Option<Arc<Mutex<TaskMetaExt>>>>,
    pub(crate) context: TaskContext,
}

impl CPU {
    const fn empty() -> Self {
        Self {
            task: RefCell::new(None),
            context: TaskContext::empty(),
        }
    }
    pub fn current(&self) -> Option<Arc<Mutex<TaskMetaExt>>> {
        self.task.borrow().as_ref().map(Arc::clone)
    }
    pub fn set_current(&self, task_meta: Arc<Mutex<TaskMetaExt>>) {
        self.task.borrow_mut().replace(task_meta);
    }
    pub fn get_idle_task_cx_ptr(&self) -> *mut TaskContext {
        &self.context as *const TaskContext as *mut _
    }
}

static CPU: CpuLocal<CPU> = CpuLocal::new(CPU::empty());

pub fn current_cpu() -> &'static CPU {
    &CPU
}

pub fn current_task() -> Option<Arc<Mutex<TaskMetaExt>>> {
    CPU.current()
}

pub fn current_tid() -> Option<usize> {
    current_task().map(|task| task.lock().tid())
}

pub fn schedule() {
    if let Some(task) = fetch_task() {
        switch_to_task(task);
    }
}

pub fn switch_to_task(task: Arc<Mutex<TaskMetaExt>>) {
    let current = current_task();
    let current_task_ctx_ptr = match current {
        Some(task) => {
            let status = task.lock().status();
            let ctx = task.lock().get_context_raw_mut_ptr();
            match status {
                TaskStatus::Ready => {
                    add_task(task);
                }
                _ => {}
            }
            ctx
        }
        None => current_cpu().get_idle_task_cx_ptr(),
    };
    let next_tid = task.lock().tid();
    task.lock().set_status(TaskStatus::Running);
    let next_task_ctx_ptr = task.lock().get_context_raw_mut_ptr();
    current_cpu().set_current(task);

    crate::domain_proxy::continuation::set_current_task_id(next_tid);
    crate::task::switch(current_task_ctx_ptr, next_task_ctx_ptr)
}
