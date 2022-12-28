    .section .text.entry
    .globl _start
_start:
    mv tp,a0
    la sp,bootstack
    addi t0,a0,1
    slli t0,t0,16
    add sp,sp,t0
    call rust_main


 sleep:
    wfi
    j sleep

    .section .bss.stack
    .global bootstack
bootstack:
    .space 4096 * 16 * 4 #
    .global bootstacktop
bootstacktop:


