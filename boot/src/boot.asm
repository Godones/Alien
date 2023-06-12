    .section .text.entry
    .globl _start
_start:
    mv tp, a0
    add t0, a0, 1
    slli t0, t0, 16
    la sp, boot_stack
    add sp, sp, t0
    call main

    .section .bss.stack
    .globl boot_stack
    .globl boot_stack_top
boot_stack:
    .globl boot_stack
    # 64K 启动栈大小 * CPU_NUMS
    .space 4096 * 16 * 8
boot_stack_top:
    .globl boot_stack_top
