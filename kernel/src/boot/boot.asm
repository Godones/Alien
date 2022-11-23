    .section .text.entry
    .globl _start
_start:
    #clear bss
	la 		t0, sbss
	la		t1, ebss
	bgeu	t0, t1, 2f
1:
	sd		zero, (t0)
	addi	t0, t0, 8
	bltu	t0, t1, 1b
2:
    # alloc stack for cpu
    # a0 == hartid
    add t0,a0,0
    slli t0,t0,12
    la sp, bootstack+0x1000
    add sp,sp,t0

    bnez a0,sleep

    j rust_main

 sleep:
    wfi
    j sleep

    .section .bss.stack
    .align 12   # page align
    .global bootstack
bootstack:
    .space 4096 * 8 #
    .global bootstacktop
bootstacktop:


