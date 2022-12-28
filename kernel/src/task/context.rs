use core::arch::asm;

/// 线程切换需要保存的上下文
///
/// 线程切换由__switch()完成，这个汇编函数不会由编译器完成寄存器保存，因此需要手动保存
#[derive(Debug, Default)]
#[repr(C)]
pub struct ThreadContext {
    ra: usize,
    sp: usize,
    s: [usize; 12],
}

impl ThreadContext {
    pub fn new(ra: usize, sp: usize) -> Self {
        Self { ra, sp, s: [0; 12] }
    }
}

#[naked]
#[no_mangle]
#[link_section = ".text"]
pub extern "C" fn __switch(current: *mut ThreadContext, next: *mut ThreadContext) -> ! {
    unsafe {
        asm!(
            "sd ra,0(a0)",
            "sd sp,8(a0)",
            "sd s0,16(a0)",
            "sd s1,24(a0)",
            "sd s2,32(a0)",
            "sd s3,40(a0)",
            "sd s4,48(a0)",
            "sd s5,56(a0)",
            "sd s6,64(a0)",
            "sd s7,72(a0)",
            "sd s8,80(a0)",
            "sd s9,88(a0)",
            "sd s10,96(a0)",
            "sd s11,104(a0)",
            "ld ra,0(a1)",
            "ld sp,8(a1)",
            "ld s0,16(a0)",
            "ld s1,24(a0)",
            "ld s2,32(a0)",
            "ld s3,40(a0)",
            "ld s4,48(a0)",
            "ld s5,56(a0)",
            "ld s6,64(a0)",
            "ld s7,72(a0)",
            "ld s8,80(a0)",
            "ld s9,88(a0)",
            "ld s10,96(a0)",
            "ld s11,104(a0)",
            "ret",
            options(noreturn)
        )
    }
}
