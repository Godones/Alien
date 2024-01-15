//! 线程切换的上下文结构
use core::arch::global_asm;

/// 线程切换需要保存的上下文
///
/// 线程切换由__switch()完成，这个汇编函数不会由编译器完成寄存器保存，因此需要手动保存
#[derive(Debug, Clone)]
#[repr(C)]
pub struct Context {
    /// ra 寄存器
    ra: usize,
    /// sp 寄存器值
    sp: usize,
    /// s0 ~ s11
    s: [usize; 12],
}

impl Context {
    /// 创建一个新的上下文，默认 s0 ~ s11 的值为 0
    pub fn new(ra: usize, sp: usize) -> Self {
        Self { ra, sp, s: [0; 12] }
    }

    /// 创建一个全为 0 的上下文
    pub const fn empty() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }
}

global_asm!(include_str!("switch.asm"));

extern "C" {
    pub fn __switch(current_task_cx_ptr: *mut Context, next_task_cx_ptr: *const Context);
}

/// 交换前后两个线程的上下文，调用 `switch.asm` 中的 `__switch`
#[inline(always)]
pub fn switch(current_task_cx_ptr: *mut Context, next_task_cx_ptr: *const Context) {
    unsafe {
        __switch(current_task_cx_ptr, next_task_cx_ptr);
    }
}
