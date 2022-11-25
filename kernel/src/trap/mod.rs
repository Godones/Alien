//todo!(使用向量中断入口而不是统一中断入口)
use core::arch::global_asm;
use riscv::register::scause::{Interrupt, Trap};
use riscv::register::stvec::TrapMode;
use riscv::register::{sie, sstatus, stvec};

mod context;
mod exception;
mod interrupt;

global_asm!(include_str!("./kernel_v.asm"));

/// 开启中断/异常
pub fn init_trap_subsystem() {
    extern "C" {
        #[allow(unused)]
        fn kernel_v();
    }
    unsafe {
        // 设置内核陷入处理
        stvec::write(kernel_v as usize, TrapMode::Direct);
        // 开启全局中断
        sstatus::set_sie();
        // 开启时钟中断
        sie::set_stimer();
    }
}

#[no_mangle]
/// 只有在内核态下才能进入这个函数
/// 避免嵌套中断发生这里不会再开启中断
pub fn kernel_trap_vector() {
    let cause = riscv::register::scause::read();
    let cause = cause.cause();
    match &cause {
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            info!("timer interrupt");
            interrupt::timer_interrupt_handler();
        }
        Trap::Exception(_) => {}
        _ => {
            panic!("unhandled trap: {:?}", cause);
        }
    }
}
