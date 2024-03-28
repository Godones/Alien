use core::arch::global_asm;

use context::TaskContext;
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
