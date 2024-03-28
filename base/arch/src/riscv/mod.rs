mod regs;

use core::arch::asm;

pub use regs::*;
use riscv::register::satp;

/// 获取当前的 hart id
pub fn hart_id() -> usize {
    let mut id: usize;
    unsafe {
        asm!(
        "mv {},tp", out(reg)id,
        );
    }
    id
}

/// 检查全局中断是否开启
pub fn is_interrupt_enable() -> bool {
    riscv::register::sstatus::read().sie()
}

/// 关闭全局中断
pub fn interrupt_disable() {
    unsafe {
        riscv::register::sstatus::clear_sie();
    }
}

/// 开启全局中断
pub fn interrupt_enable() {
    unsafe {
        riscv::register::sstatus::set_sie();
    }
}

/// 开启外部中断
pub fn external_interrupt_enable() {
    unsafe {
        riscv::register::sie::set_sext();
    }
}

/// 开启软件中断
pub fn software_interrupt_enable() {
    unsafe {
        riscv::register::sie::set_ssoft();
    }
}

/// 关闭外部中断
pub fn external_interrupt_disable() {
    unsafe {
        riscv::register::sie::clear_sext();
    }
}

/// 开启时钟中断
pub fn timer_interrupt_enable() {
    unsafe {
        riscv::register::sie::set_stimer();
    }
}

/// 读取时钟
pub fn read_timer() -> usize {
    riscv::register::time::read()
}

/// 激活页表模式
pub fn activate_paging_mode(root_ppn: usize) {
    unsafe {
        sfence_vma_all();
        satp::set(satp::Mode::Sv39, 0, root_ppn);
        sfence_vma_all();
    }
}

pub fn sfence_vma_all() {
    unsafe { riscv::asm::sfence_vma_all() }
}

pub fn allow_access_user_memory() {
    unsafe {
        riscv::register::sstatus::set_sum();
    }
}
