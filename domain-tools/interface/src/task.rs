use crate::Basic;

pub trait TaskDomain: Basic {
    fn run(&self);
    fn current_task_trap_frame_ptr(&self) -> usize;
    fn current_task_satp(&self) -> usize;
}
