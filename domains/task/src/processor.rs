use crate::task::Task;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use context::{TaskContext, TrapFrame};
use ksync::{Mutex, MutexGuard};
use spin::lazy::Lazy;

#[derive(Debug, Clone)]
pub struct CPU {
    pub task: Option<Arc<Task>>,
    pub context: TaskContext,
}

impl CPU {
    const fn empty() -> Self {
        Self {
            task: None,
            context: TaskContext::empty(),
        }
    }
    pub fn take_current(&mut self) -> Option<Arc<Task>> {
        self.task.take()
    }
    pub fn current(&self) -> Option<Arc<Task>> {
        self.task.clone()
    }
    pub fn get_idle_task_cx_ptr(&mut self) -> *mut TaskContext {
        &mut self.context as *mut TaskContext
    }
}

static CPU: Mutex<CPU> = Mutex::new(CPU::empty());

pub fn current_cpu() -> MutexGuard<'static, CPU> {
    CPU.lock()
}

pub fn current_task() -> Option<Arc<Task>> {
    CPU.lock().current()
}

pub fn take_current_task() -> Option<Arc<Task>> {
    CPU.lock().take_current()
}

pub fn current_user_token() -> usize {
    let task = current_task().unwrap();
    task.token()
}

pub fn current_trap_frame() -> &'static mut TrapFrame {
    let task = current_task().unwrap();
    task.trap_frame()
}

pub fn current_trap_frame_ptr() -> usize {
    let task = current_task().unwrap();
    task.trap_frame_ptr()
}

static GLOBAL_TASK_MANAGER: Lazy<Arc<Mutex<VecDeque<Arc<Task>>>>> =
    Lazy::new(|| Arc::new(Mutex::new(VecDeque::new())));

pub fn add_task(task: Arc<Task>) {
    GLOBAL_TASK_MANAGER.lock().push_back(task);
}

pub fn pick_next_task() -> Option<Arc<Task>> {
    GLOBAL_TASK_MANAGER.lock().pop_front()
}
