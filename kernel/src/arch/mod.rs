use core::arch::asm;

use self::riscv::register::sie;
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

#[cfg(any(feature = "vf2", feature = "hifive"))]
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

/// 检查全局中断是否开启
pub fn is_interrupt_enable() -> bool {
    sstatus::read().sie()
}

/// 关闭全局中断
pub fn interrupt_disable() {
    unsafe {
        sstatus::clear_sie();
    }
}

/// 开启全局中断
pub fn interrupt_enable() {
    unsafe {
        sstatus::set_sie();
    }
}

/// 开启外部中断
pub fn external_interrupt_enable() {
    unsafe {
        sie::set_sext();
    }
}

/// 开启软件中断
pub fn software_interrupt_enable() {
    unsafe {
        sie::set_ssoft();
    }
}

/// 关闭外部中断
pub fn external_interrupt_disable() {
    unsafe {
        sie::clear_sext();
    }
}

/// 开启时钟中断
pub fn timer_interrupt_enable() {
    unsafe {
        sie::set_stimer();
    }
}

/// 读取时钟
pub fn read_timer() -> usize {
    riscv::register::time::read()
}

#[macro_export]
macro_rules! write_csr {
    ($csr:ident, $val:expr) => {
        unsafe {
            asm!(concat!("csrw ", stringify!($csr), ", {}"), in(reg) $val);
        }
    };
}
