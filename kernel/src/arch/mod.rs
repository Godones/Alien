use core::arch::asm;

use self::riscv::register::{scause, sie};
use self::riscv::sstatus;

pub mod riscv;

#[cfg(feature = "qemu")]
pub fn hart_id() -> usize {
    let mut id: usize;
    unsafe {
        asm!(
        "mv {},tp", out(reg)id,
        );
    }
    id
}

#[cfg(any(feature = "vf2", feature = "sifive"))]
pub fn hart_id() -> usize {
    let mut id: usize;
    unsafe {
        asm!(
        "mv {},tp", out(reg)id,
        );
    }
    id -= 1;
    id
}


pub fn is_interrupt_enable() -> bool {
    sstatus::read().sie()
}

pub fn interrupt_disable() {
    unsafe {
        sstatus::clear_sie();
    }
}

/// enable the global interrupt
pub fn interrupt_enable() {
    unsafe {
        sstatus::set_sie();
    }
}

pub fn external_interrupt_enable() {
    unsafe {
        // 开启外部中断
        sie::set_sext();
    }
}

pub fn software_interrupt_enable() {
    unsafe {
        // 开启软件中断
        sie::set_ssoft();
    }
}

pub fn external_interrupt_disable() {
    unsafe {
        // 关闭外部中断
        sie::clear_sext();
    }
}

pub fn timer_interrupt_enable() {
    unsafe {
        // 开启时钟中断
        sie::set_stimer();
    }
}

pub fn interrupt_cause() -> usize {
    scause::read().bits()
}

pub fn read_timer() -> usize {
    riscv::register::time::read()
}

pub fn set_timer(addition: usize) {
    crate::sbi::set_timer(read_timer() + addition)
}

#[macro_export]
macro_rules! write_csr {
    ($csr:ident, $val:expr) => {
        unsafe {
            asm!(concat!("csrw ", stringify!($csr), ", {}"), in(reg) $val);
        }
    };
}
