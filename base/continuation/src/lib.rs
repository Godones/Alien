#![no_std]

extern crate alloc;

use alloc::{collections::BTreeMap, vec::Vec};
use core::{arch::global_asm, sync::atomic::AtomicUsize};

use config::CPU_NUM;
use ksync::Mutex;
use log::{debug, info};
use spin::Lazy;

#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct Continuation {
    // all registers
    pub regs: [usize; 32],
    // function ptr
    pub func: usize,
}

impl Continuation {
    pub fn empty() -> Self {
        Self {
            func: 0,
            regs: [0; 32],
        }
    }
}

static CTID: [AtomicUsize; CPU_NUM] = [AtomicUsize::new(0); CPU_NUM];

// when tid == usize::MAX, it means we are in the idle loop
pub fn set_current_task_id(tid: usize) {
    let cpu_id = arch::hart_id();
    CTID[cpu_id].store(tid, core::sync::atomic::Ordering::Relaxed);
}

static TASK_CONTEXT: Lazy<Mutex<BTreeMap<usize, Vec<Continuation>>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));

/// Register a continuation for the current thread in the current domain.
pub fn register_continuation(context: &Continuation) {
    let mut binding = TASK_CONTEXT.lock();
    let mut new_context = context.clone();
    new_context.regs[2] += 33 * 8; // sp += 33 * 8

    let cpu_id = arch::hart_id();
    let tid = CTID[cpu_id].load(core::sync::atomic::Ordering::Relaxed);

    let context_list = binding.entry(tid).or_insert(Vec::new());
    context_list.push(new_context);
    debug!(
        "<register_continuation>: cpu:{} ,tid:{}, len:{}",
        cpu_id,
        tid,
        context_list.len()
    );
}

pub fn pop_continuation() -> Option<Continuation> {
    let cpu_id = arch::hart_id();
    let tid = CTID[cpu_id].load(core::sync::atomic::Ordering::Relaxed);
    let mut binding = TASK_CONTEXT.lock();
    let context_list = binding.get_mut(&tid).unwrap();
    debug!(
        "<pop_continuation>: cpu:{} ,tid:{}, len:{}",
        cpu_id,
        tid,
        context_list.len()
    );
    context_list.pop()
}

#[allow(unused)]
pub fn clear_continuation() {
    let cpu_id = arch::hart_id();
    let tid = CTID[cpu_id].load(core::sync::atomic::Ordering::Relaxed);
    let mut binding = TASK_CONTEXT.lock();
    let context_list = binding.get_mut(&tid).unwrap();
    assert_eq!(context_list.len(), 0);
    info!(
        "<clear_continuation>: cpu:{} ,tid:{}, len:{}",
        cpu_id,
        tid,
        context_list.len()
    );
}

pub fn unwind() -> ! {
    let continuation = pop_continuation().unwrap();
    platform::println!("unwind, continuation: {:#x?}", &continuation);
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
    ld x1, 1*8(a0)
    ld x2, 2*8(a0)
    ld x3, 3*8(a0)
    ld x4, 4*8(a0)
    ld x5, 5*8(a0)
    ld x6, 6*8(a0)
    ld x7, 7*8(a0)
    ld x8, 8*8(a0)
    ld x9, 9*8(a0)
    # ld x10, 10*8(a0)
    ld x11, 11*8(a0)
    ld x12, 12*8(a0)
    ld x13, 13*8(a0)
    ld x14, 14*8(a0)
    ld x15, 15*8(a0)
    ld x16, 16*8(a0)
    ld x17, 17*8(a0)
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
    ld x28, 28*8(a0)
    ld x29, 29*8(a0)
    ld x30, 30*8(a0)
    ld x31, 31*8(a0)

    mv gp, a0
    ld a0, 10*8(gp)  # a0==x10
    ld gp, 32*8(gp)  # gp -> func
    jr gp

    "#
);
