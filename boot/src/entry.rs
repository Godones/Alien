//! 内核汇编入口
use core::arch::asm;

use kernel::config::{CPU_NUM, STACK_SIZE};

#[link_section = ".bss.stack"]
pub static mut STACK: [u8; STACK_SIZE * CPU_NUM] = [0; STACK_SIZE * CPU_NUM];

/// 内核入口
///
/// 用于初始化内核的栈空间，并关闭中断
#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
extern "C" fn _start() {
    unsafe {
        asm!("\
        mv tp, a0
        csrw sscratch, a1
        add t0, a0, 1
        slli t0, t0, 16
        la sp, {boot_stack}
        add sp, sp, t0
        call main
        ",
        boot_stack = sym STACK,
        options(noreturn)
        );
    }
}

/// 获取设备树地址
/// 在开始阶段，内核会将设备树的地址保存在sscratch寄存器中
#[allow(unused)]
#[inline]
pub fn device_tree_addr() -> usize {
    let mut res: usize;
    unsafe {
        asm!(
        " csrr {}, sscratch",
        out(reg) res,
        )
    }
    res
}
