#![no_std]

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
