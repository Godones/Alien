.altmacro
.macro KSAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro KLOAD_GP n
    ld x\n, \n*8(sp)
.endm


.section .text
.globl kernel_v
.align 2
kernel_v:
    addi sp, sp, -34*8
    sd x1, 1*8(sp)
    sd x3, 3*8(sp)
    .set n, 5
    .rept 27
        KSAVE_GP %n
        .set n, n+1
    .endr
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    mv a0, sp
    # call the C trap handler in trap.c
    csrr t2, sscratch
    jalr t2


     ld t0, 32*8(sp)
     ld t1, 33*8(sp)
     csrw sstatus, t0
     csrw sepc, t1
     ld x1, 1*8(sp)
     ld x3, 3*8(sp)
     .set n, 5
     .rept 27
         KLOAD_GP %n
         .set n, n+1
     .endr
     addi sp, sp, 34*8
     sret
