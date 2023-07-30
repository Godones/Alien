use core::arch::asm;

use kernel::config::{CPU_NUM, STACK_SIZE};

#[link_section = ".bss.stack"]
static mut STACK: [u8; STACK_SIZE * CPU_NUM] = [0; STACK_SIZE * CPU_NUM];

#[cfg(any(feature = "vf2", feature = "hifive"))]
#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
extern "C" fn _start() {
    unsafe {
        asm!("\
        mv tp, a0
        csrw sscratch, a1
        csrci sstatus, 0x02
        csrw sie, zero
        add t0, a0, 0
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

#[cfg(feature = "qemu")]
#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
extern "C" fn _start() {
    unsafe {
        asm!("\
        mv tp, a0
        csrw sscratch, a1
        csrci sstatus, 0x02
        csrw sie, zero
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
