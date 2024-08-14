use alloc::sync::Arc;
use core::hint::spin_loop;

use basic::{
    arch::{hart_id, CpuLocal},
    sync::Mutex,
};
use config::CPU_NUM;
use task_meta::{TaskContext, TaskStatus};

use crate::task::{
    resource::TaskMetaExt,
    scheduler::{add_task, fetch_task},
};

#[derive(Debug, Clone)]
pub struct Cpu {
    pub(crate) task: Option<Arc<Mutex<TaskMetaExt>>>,
    pub(crate) context: TaskContext,
}

impl Cpu {
    const fn empty() -> Self {
        Self {
            task: None,
            context: TaskContext::empty(),
        }
    }
    pub fn current(&self) -> Option<Arc<Mutex<TaskMetaExt>>> {
        self.task.as_ref().map(Arc::clone)
    }
    pub fn take_current(&mut self) -> Option<Arc<Mutex<TaskMetaExt>>> {
        self.task.take()
    }
    pub fn set_current(&mut self, task_meta: Arc<Mutex<TaskMetaExt>>) {
        self.task.replace(task_meta);
    }
    pub fn get_idle_task_cx_ptr(&self) -> *mut TaskContext {
        &self.context as *const TaskContext as *mut _
    }
}

const CPU_ONE: CpuLocal<Cpu> = CpuLocal::new(Cpu::empty());
static CPUS: [CpuLocal<Cpu>; CPU_NUM] = [CPU_ONE; CPU_NUM];

pub fn current_cpu() -> &'static mut Cpu {
    CPUS[hart_id()].as_mut()
}

pub fn current_task() -> Option<Arc<Mutex<TaskMetaExt>>> {
    CPUS[hart_id()].current()
}

pub fn current_tid() -> Option<usize> {
    current_task().map(|task| task.lock().tid())
}

pub fn take_current_task() -> Option<Arc<Mutex<TaskMetaExt>>> {
    CPUS[hart_id()].as_mut().take_current()
}

pub fn schedule() {
    let cpu = current_cpu();
    let current_task = current_task().unwrap();
    let task_context = current_task.lock().get_context_raw_mut_ptr();
    drop(current_task);
    let cpu_context = cpu.get_idle_task_cx_ptr();
    crate::task::switch(task_context, cpu_context);
}

pub fn cpu_loop() {
    loop {
        let cpu = current_cpu();
        let current_task = take_current_task();
        let tid = match current_task {
            Some(task) => {
                let tid = task.lock().tid();
                let status = task.lock().status();
                match status {
                    TaskStatus::Ready => {
                        add_task(task);
                    }
                    TaskStatus::Zombie => {
                        task.lock().set_status(TaskStatus::Terminated);
                    }
                    _ => {}
                }
                Some(tid)
            }
            None => None,
        };
        if let Some(next_task) = fetch_task() {
            next_task.lock().set_status(TaskStatus::Running);
            let next_task_ctx_ptr = next_task.lock().get_context_raw_mut_ptr();
            log::warn!(
                "[tid: {:?}] switch to task {:?}",
                tid,
                next_task.lock().tid()
            );
            cpu.set_current(next_task);
            let cpu_context = cpu.get_idle_task_cx_ptr();
            crate::task::switch(cpu_context, next_task_ctx_ptr)
        } else {
            spin_loop();
        }
    }
}
