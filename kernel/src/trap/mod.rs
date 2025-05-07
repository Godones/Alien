use core::arch::{asm, global_asm};

use ::interrupt::{external_interrupt_handler, record_irq};
use arch::{
    external_interrupt_enable, interrupt_disable, interrupt_enable, is_interrupt_enable,
    timer_interrupt_enable,
};
use bit_field::BitField;
use config::TRAMPOLINE;
use constants::{
    signal::{SignalNumber, SIGNAL_RETURN_TRAP},
    time::TimerType,
    AlienError,
};
pub use context::{CommonTrapFrame, TrapFrame};
pub use exception::trap_common_read_file;
use riscv::register::{
    scause::{Exception, Interrupt, Trap},
    sepc, sstatus,
    sstatus::SPP,
    stval, stvec,
    stvec::TrapMode,
};

use crate::{
    ipc::{send_signal, signal_handler, signal_return, solve_futex_wait},
    task::{current_task, current_trap_frame, current_user_token, do_exit, do_suspend},
    time::{check_timer_queue, set_next_trigger_in_kernel},
    trap::context::KTrapFrame,
};

mod context;
mod exception;
mod interrupt;

global_asm!(include_str!("./kernel_v.asm"));
global_asm!(include_str!("./trampoline.asm"));

extern "C" {
    fn kernel_v();
    fn user_v();
    fn user_r();
}

#[no_mangle]
/// set the new addr of __restore asm function in TRAMPOLINE page,
/// set the reg a0 = trap_cx_ptr, reg a1 = phy addr of usr page table,
/// finally, jump to new addr of __restore asm function
pub fn trap_return() -> ! {
    signal_handler();
    interrupt_disable();
    set_user_trap_entry();
    let trap_frame = current_trap_frame();
    let sstatues = trap_frame.get_status();
    let enable = sstatues.0.get_bit(5);
    let sie = sstatues.0.get_bit(1);
    assert!(enable);
    assert!(!sie);
    let trap_cx_ptr = current_task().unwrap().trap_frame_ptr();
    let user_satp = current_user_token();
    let restore_va = user_r as usize - user_v as usize + TRAMPOLINE;
    unsafe {
        asm!(
        "fence.i",
        "jr {restore_va}",
        restore_va = in(reg) restore_va,
        in("a0") trap_cx_ptr,
        in("a1") user_satp,
        options(noreturn)
        )
    }
}

/// 设置用户态 trap 处理例程的入口点
#[inline]
fn set_user_trap_entry() {
    unsafe {
        stvec::write(TRAMPOLINE, TrapMode::Direct);
    }
}

/// 设置内核态 trap 处理例程的入口点
#[inline]
pub fn set_kernel_trap_entry() {
    unsafe {
        stvec::write(kernel_v as usize, TrapMode::Direct);
    }
}

/// 开启中断/异常
pub fn init_trap_subsystem() {
    println!("++++ setup interrupt ++++");
    set_kernel_trap_entry();
    external_interrupt_enable();
    timer_interrupt_enable();
    interrupt_enable();
    let enable = is_interrupt_enable();
    println!("++++ setup interrupt done, enable:{:?} ++++", enable);
}

pub trait TrapHandler {
    fn do_user_handle(&self);
    fn do_kernel_handle(&self, ktrap_frame: &'static mut KTrapFrame);
}

impl TrapHandler for Trap {
    /// 用户态下的 trap 例程
    fn do_user_handle(&self) {
        let stval = stval::read();
        let sepc = sepc::read();
        trace!("trap :{:?}", self);
        match self {
            Trap::Exception(Exception::UserEnvCall) => {
                exception::syscall_exception_handler();
            }
            Trap::Exception(Exception::StoreFault)
            | Trap::Exception(Exception::LoadFault)
            | Trap::Exception(Exception::InstructionFault)
            | Trap::Exception(Exception::IllegalInstruction) => {
                error!(
                    "[User] {:?} in application,stval:{:#x?} sepc:{:#x?}",
                    self, stval, sepc
                );
                let task = current_task().unwrap();
                send_signal(task.get_tid() as usize, SignalNumber::SIGSEGV as usize)
            }
            Trap::Exception(Exception::StorePageFault)
            | Trap::Exception(Exception::LoadPageFault) => {
                let task = current_task().unwrap();
                let tid = task.get_tid();
                warn!(
                    "[User][tid:{}] {:?} in application,stval:{:#x?} sepc:{:#x?}",
                    tid, self, stval, sepc
                );
                let res = exception::page_exception_handler(self.clone(), stval);
                if res.is_err() {
                    error!(
                        "[User] {:?} in application,stval:{:#x?} sepc:{:#x?}",
                        self, stval, sepc
                    );
                    let err = res.err().unwrap();
                    if err == AlienError::EAGAIN {
                        // println!("thread need wait");
                        do_suspend();
                    } else if err == AlienError::EPERM {
                        do_exit(-1, 0);
                    } else {
                        send_signal(tid as usize, SignalNumber::SIGSEGV as usize)
                    }
                }
            }
            Trap::Exception(Exception::InstructionPageFault) => {
                trace!(
                    "[User] {:?} in application,stval:{:#x?} sepc:{:#x?}",
                    self,
                    stval,
                    sepc
                );
                if stval == SIGNAL_RETURN_TRAP {
                    // 当作调用了 sigreturn 一样
                    let cx = current_trap_frame();
                    cx.regs()[10] = signal_return() as usize;
                    return;
                }

                let res = exception::page_exception_handler(self.clone(), stval);
                if res.is_err() {
                    error!(
                        "[User] {:?} in application,stval:{:#x?} sepc:{:#x?}",
                        self, stval, sepc
                    );
                    let task = current_task().unwrap();
                    send_signal(task.get_tid() as usize, SignalNumber::SIGSEGV as usize)
                }
            }
            Trap::Interrupt(Interrupt::SupervisorTimer) => {
                trace!("[User] timer interrupt");
                interrupt::timer_interrupt_handler();
            }
            Trap::Interrupt(Interrupt::SupervisorExternal) => {
                trace!("external interrupt");
                external_interrupt_handler();
            }
            Trap::Exception(Exception::Breakpoint) => {
                // breakpoint
                let trap_frame = current_trap_frame();
                exception::ebreak_handler(CommonTrapFrame::User(trap_frame));
            }
            _ => {
                panic!(
                    "unhandled trap: {:?}, stval: {:?}, sepc: {:x}",
                    self, stval, sepc
                );
            }
        }
    }

    /// 内核态下的 trap 例程
    fn do_kernel_handle(&self, ktrap: &'static mut KTrapFrame) {
        let stval = stval::read();
        let sepc = sepc::read();
        match self {
            Trap::Interrupt(Interrupt::SupervisorTimer) => {
                trace!("[kernel] timer interrupt");
                record_irq(1);
                check_timer_queue();
                solve_futex_wait();
                set_next_trigger_in_kernel();
            }
            Trap::Exception(Exception::StorePageFault) => {
                debug!(
                    "[kernel] {:?} in kernel, stval:{:#x?} sepc:{:#x?}",
                    self, stval, sepc
                );
                {
                    let phy = mem::query_kernel_space(stval);
                    debug!("physical address: {:#x?}", phy);
                }
            }
            Trap::Exception(Exception::Breakpoint) => {
                exception::ebreak_handler(CommonTrapFrame::Kernel(ktrap));
            }
            Trap::Exception(_) => {
                panic!(
                    "unhandled trap: {:?}, stval: {:#x?}, sepc: {:#x}",
                    self, stval, sepc,
                )
            }
            Trap::Interrupt(Interrupt::SupervisorExternal) => {
                external_interrupt_handler();
            }
            _ => {
                panic!(
                    "unhandled trap: {:?}, stval: {:?}, sepc: {:x}",
                    self, stval, sepc
                )
            }
        }
    }
}

/// 用户态陷入处理
#[no_mangle]
pub fn user_trap_vector() {
    let sstatus = sstatus::read();
    let spp = sstatus.spp();
    if spp == SPP::Supervisor {
        panic!("user_trap_vector: spp == SPP::Supervisor");
    }
    {
        let task = current_task().expect("user_trap_vector: current_task is none");
        // update process statistics
        task.access_inner().update_user_mode_time();
        check_task_timer_expired();
    }
    set_kernel_trap_entry();
    let cause = riscv::register::scause::read();
    let cause = cause.cause();
    cause.do_user_handle();
    let task = current_task().unwrap();
    if cause != Trap::Interrupt(Interrupt::SupervisorTimer) {
        // update process statistics
        task.access_inner().update_kernel_mode_time();
    }
    task.access_inner().update_timer();
    check_task_timer_expired();
    crate::task::check_exit_group();
    trap_return();
}

/// 用于检查进程的计时器是否超时。如果超时则会重置计时器，并按照计时器类型向进程发送信号。
pub fn check_task_timer_expired() {
    let task = current_task().unwrap();
    let timer_expired = task.access_inner().check_timer_expired();
    let tid = task.get_tid() as usize;
    if timer_expired.is_some() {
        error!("timer expired: {:?}", timer_expired);
        let timer_type = timer_expired.unwrap();
        match timer_type {
            TimerType::REAL => send_signal(tid, SignalNumber::SIGALRM as usize),
            TimerType::VIRTUAL => send_signal(tid, SignalNumber::SIGVTALRM as usize),
            TimerType::PROF => send_signal(tid, SignalNumber::SIGPROF as usize),
            _ => {
                panic!("timer type error");
            }
        };
    }
}

/// 只有在内核态下才能进入这个函数
/// 避免嵌套中断发生这里不会再开启中断
#[no_mangle]
extern "C" fn kernel_trap_vector(ktrap: &'static mut KTrapFrame) {
    let sstatus = sstatus::read();
    let spp = sstatus.spp();
    if spp == SPP::User {
        panic!("kernel_trap_vector: spp == SPP::User");
    }
    let enable = is_interrupt_enable();
    assert!(!enable);
    let cause = riscv::register::scause::read().cause();
    cause.do_kernel_handle(ktrap)
}
