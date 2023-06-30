.altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm
    .section .text.trampoline
    .globl user_v
    .globl user_r
    .align 2
user_v:
    csrrw sp, sscratch, sp
    # now sp->*TrapContext in user space, sscratch->user stack
    # save other general purpose registers
    sd x1, 1*8(sp)
    # skip sp(x2), we will save it later
    # save x3~x31
    .set n, 3
    .rept 29
        SAVE_GP %n
        .set n, n+1
    .endr
    # we can use t0/t1/t2 freely, because they have been saved in TrapContext
    csrr t0, sstatus
    csrr t1, sepc
    sd t1, 32*8(sp)
    sd t0, 37*8(sp)
    # read user stack from sscratch and save it in TrapContext
    csrr t2, sscratch
    sd t2, 2*8(sp)
    # load kernel_satp into t0
    ld t0, 33*8(sp)
    # load trap_handler into t1
    ld t1, 35*8(sp)
    # load tp
    ld tp,36*8(sp)
    # move to kernel_sp
    ld sp, 34*8(sp)

    # load hartid into tp(x4)
    # ld tp, 36*8(tp)
    # 保证用户态缓存全部刷新到内存
    sfence.vma
    # switch to kernel space
    csrw satp, t0
    sfence.vma
    # jump to trap_handler
    jr t1

user_r:
    # a0: *TrapContext in user space(Constant); a1: user space token
    # switch to user space
    sfence.vma
    csrw satp, a1
    sfence.vma
    csrw sscratch, a0
    mv sp, a0
    # now sp points to TrapContext in user space, start restoring based on it
    # restore sstatus/sepc
    ld t1, 32*8(sp)
    ld t0, 37*8(sp)
    csrw sepc, t1
    csrw sstatus, t0
    # restore general purpose registers except x0/sp
    ld x1, 1*8(sp)
    .set n, 3
    .rept 29
        LOAD_GP %n
        .set n, n+1
    .endr
    # back to user stack
    ld sp, 2*8(sp)
    sret