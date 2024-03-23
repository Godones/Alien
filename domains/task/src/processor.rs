use crate::task::Task;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use basic::arch::CpuLocal;
use context::{TaskContext, TrapFrame};
use core::cell::RefCell;
use ksync::Mutex;
use spin::lazy::Lazy;

#[derive(Debug, Clone)]
pub struct CPU {
    task: RefCell<Option<Arc<Task>>>,
    context: TaskContext,
}

impl CPU {
    const fn empty() -> Self {
        Self {
            task: RefCell::new(None),
            context: TaskContext::empty(),
        }
    }
    pub fn take_current(&self) -> Option<Arc<Task>> {
        self.task.borrow_mut().take()
    }
    pub fn current(&self) -> Option<Arc<Task>> {
        self.task.borrow().clone()
    }
    pub fn set_current(&self, task: Arc<Task>) {
        self.task.borrow_mut().replace(task);
    }
    pub fn get_idle_task_cx_ptr(&self) -> *mut TaskContext {
        &self.context as *const TaskContext as *mut _
    }
}

static CPU: CpuLocal<CPU> = CpuLocal::new(CPU::empty());

pub fn current_cpu() -> &'static CPU {
    &CPU
}

pub fn current_task() -> Option<Arc<Task>> {
    CPU.current()
}

pub fn take_current_task() -> Option<Arc<Task>> {
    CPU.take_current()
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
