use core::arch::naked_asm;

use config::{CPU_NUM, STACK_SIZE, STACK_SIZE_BITS};

#[link_section = ".bss.stack"]
static mut STACK: [u8; STACK_SIZE * CPU_NUM] = [0; STACK_SIZE * CPU_NUM];

/// 内核入口
///
/// 用于初始化内核的栈空间，并关闭中断
#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
extern "C" fn _start() {
    unsafe {
        naked_asm!("\
        mv tp, a0
        mv gp, a1
        add t0, a0, 1
        slli t0, t0, {stack_size_bits}
        la sp, {boot_stack}
        add sp, sp, t0
        mv a0, tp
        mv a1, gp
        call {platform_init}
        ",
        stack_size_bits = const STACK_SIZE_BITS,
        boot_stack = sym STACK,
        platform_init = sym crate::platform_init,
        );
    }
}

#[naked]
#[no_mangle]
extern "C" fn _start_secondary() {
    unsafe {
        naked_asm!("\
        mv tp, a0
        mv gp, a1
        add t0, a0, 1
        slli t0, t0, {stack_size_bits}
        la sp, {boot_stack}
        add sp, sp, t0
        mv a0, tp
        mv a1, gp
        call main
        ",
        stack_size_bits = const STACK_SIZE_BITS,
        boot_stack = sym STACK,
        );
    }
}
