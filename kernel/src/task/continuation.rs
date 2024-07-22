use alloc::vec::Vec;
use core::arch::global_asm;

use log::debug;

use crate::task::processor::{current_cpu, current_task};

#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct Continuation {
    // all registers
    pub regs: [usize; 32],
    // function ptr
    pub func: usize,
}

#[derive(Debug, Clone)]
pub struct ContinuationManager {
    continuation: Vec<Continuation>,
}

impl ContinuationManager {
    pub const fn new() -> Self {
        Self {
            continuation: Vec::new(),
        }
    }
    pub fn register_continuation(&mut self, context: Continuation) {
        self.continuation.push(context);
    }
    pub fn pop_continuation(&mut self) -> Option<Continuation> {
        self.continuation.pop()
    }
    pub fn continuation_len(&self) -> usize {
        self.continuation.len()
    }
}

/// Register a continuation for the current thread in the current domain.
pub fn register_continuation(context: &Continuation) {
    let mut new_context = context.clone();
    new_context.regs[2] += 33 * 8; // sp += 33 * 8

    let cpu_id = arch::hart_id();
    let task = current_task();
    let (len, tid) = if let Some(task) = task {
        let mut guard = task.lock();
        guard.continuation.register_continuation(new_context);
        (guard.continuation.continuation_len(), Some(guard.tid()))
    } else {
        let cpu = current_cpu();
        cpu.continuation.register_continuation(new_context);
        (cpu.continuation.continuation_len(), None)
    };
    debug!(
        "<register_continuation>: cpu:{} ,tid:{:?}, len:{}",
        cpu_id, tid, len
    );
}

pub fn pop_continuation() -> Option<Continuation> {
    let cpu_id = arch::hart_id();
    let task = current_task();
    let (continuation, len, tid) = if let Some(task) = task {
        let mut guard = task.lock();
        (
            guard.continuation.pop_continuation(),
            guard.continuation.continuation_len(),
            Some(guard.tid()),
        )
    } else {
        let cpu = current_cpu();
        (
            cpu.continuation.pop_continuation(),
            cpu.continuation.continuation_len(),
            None,
        )
    };
    debug!(
        "<pop_continuation>: cpu:{} ,tid:{:?}, len:{}",
        cpu_id, tid, len
    );
    continuation
}

pub fn unwind() -> ! {
    let continuation = pop_continuation().unwrap();
    platform::println!("unwind continuation");
    unsafe { __unwind(&continuation) }
}

extern "C" {
    fn __unwind(continuation: &Continuation) -> !;
}

global_asm!(
    r#"
    .section .text
    .global __unwind
    .type __unwind, @function
__unwind:
    ld x1, 1*8(a0)      #ra
    ld x2, 2*8(a0)      #sp
    ld x8, 8*8(a0)
    ld x9, 9*8(a0)
    # ld x10, 10*8(a0)
    ld x18, 18*8(a0)
    ld x19, 19*8(a0)
    ld x20, 20*8(a0)
    ld x21, 21*8(a0)
    ld x22, 22*8(a0)
    ld x23, 23*8(a0)
    ld x24, 24*8(a0)
    ld x25, 25*8(a0)
    ld x26, 26*8(a0)
    ld x27, 27*8(a0)

    mv gp, a0
    ld a0, 10*8(gp)     # a0 == x10
    ld gp, 32*8(gp)     # gp -> func
    jr gp

    "#
);



#[no_mangle]
pub extern "C" fn register_cont(cont: &Continuation) {
    register_continuation(cont)
}
