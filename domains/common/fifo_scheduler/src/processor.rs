use alloc::sync::Arc;
use core::{cell::RefCell, hint::spin_loop};

use basic::{arch::CpuLocal, sync::Mutex};
use task_meta::{TaskContext, TaskMeta, TaskStatus};

use crate::scheduler::{add_task, fetch_task};

#[derive(Debug, Clone)]
pub struct CPU {
    task: RefCell<Option<Arc<Mutex<TaskMeta>>>>,
    context: TaskContext,
}

impl CPU {
    const fn empty() -> Self {
        Self {
            task: RefCell::new(None),
            context: TaskContext::empty(),
        }
    }
    pub fn take_current(&self) -> Option<Arc<Mutex<TaskMeta>>> {
        self.task.borrow_mut().take()
    }
    pub fn current(&self) -> Option<Arc<Mutex<TaskMeta>>> {
        self.task.borrow().as_ref().map(Arc::clone)
    }
    pub fn set_current(&self, task_meta: Arc<Mutex<TaskMeta>>) {
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

pub fn current_task() -> Option<Arc<Mutex<TaskMeta>>> {
    CPU.current()
}

pub fn take_current_task() -> Option<Arc<Mutex<TaskMeta>>> {
    CPU.take_current()
}

pub fn run_task() -> ! {
    loop {
        if let Some(task) = fetch_task() {
            // update state to running
            task.lock().set_status(TaskStatus::Running);
            // get the process context
            let context = task.lock().get_context_raw_ptr();
            let tid = task.lock().tid();
            let cpu = current_cpu();
            // basic::println!("switch to task: {:?}", task.pid());
            cpu.set_current(task);
            // switch to the process context
            let cpu_context = cpu.get_idle_task_cx_ptr();
            // log::warn!("switch to task: {:?}", tid);
            basic::task::switch(cpu_context, context, tid);
        } else {
            spin_loop();
        }
    }
}

pub fn switch_to_cpu(task: Arc<Mutex<TaskMeta>>) {
    let context = task.lock().get_context_raw_mut_ptr();
    let status = task.lock().status();
    match status {
        TaskStatus::Waiting => {
            drop(task);
        }
        TaskStatus::Zombie => {}
        _ => {
            // println!("add task to scheduler");
            add_task(task);
        }
    }
    let cpu = current_cpu();
    let cpu_context = cpu.get_idle_task_cx_ptr();
    basic::task::switch(context, cpu_context, usize::MAX);
}
