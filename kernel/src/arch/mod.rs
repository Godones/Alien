use core::arch::asm;
use riscv::register::{scause, sie, sstatus};
pub fn hart_id() -> usize {
    let id: usize;
    unsafe {
        asm!(
        "mv {},tp", out(reg)id
        );
    }
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

pub fn interrupt_enable() {
    unsafe {
        sstatus::set_sie();
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
