use alloc::sync::Arc;
use core::arch::global_asm;

use context::TaskContext;
use interface::TaskDomain;
use spin::Once;

global_asm!(include_str!("switch.asm"));

extern "C" {
    fn __switch(now: *mut TaskContext, next: *const TaskContext);
}

/// 交换前后两个线程的上下文，调用 `switch.asm` 中的 `__switch`
#[inline(always)]
pub fn switch(now: *mut TaskContext, next: *const TaskContext) {
    unsafe {
        __switch(now, next);
    }
}

pub static TASK_DOMAIN: Once<Arc<dyn TaskDomain>> = Once::new();

#[macro_export]
macro_rules! task_domain {
    () => {
        crate::task::TASK_DOMAIN
            .get()
            .expect("task domain not init")
    };
}

pub fn register_task_domain(task_domain: Arc<dyn TaskDomain>) {
    TASK_DOMAIN.call_once(|| task_domain);
}

pub fn run_task() {
    task_domain!().run().unwrap()
}
