#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use core::arch::global_asm;
use ksync::Mutex;
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
    pub fn from_raw_ptr(ptr: *mut u8) -> &'static mut Self {
        unsafe { &mut *(ptr as *mut Self) }
    }
}

static TASK_CONTEXT: Lazy<Mutex<Vec<Continuation>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// Register a continuation for the current thread in the current domain.
pub fn register_continuation(context: &Continuation) {
    // info!("[register_continuation]: {:#x?}", context);
    let mut binding = TASK_CONTEXT.lock();
    let mut new_context = context.clone();
    new_context.regs[2] += 33 * 8; // sp += 33 * 8
    binding.push(new_context);
    if context.func != 0 {
        // platform::system_shutdown();
    }
}

fn pop_continuation() -> Option<Continuation> {
    let mut binding = TASK_CONTEXT.lock();
    binding.pop()
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
