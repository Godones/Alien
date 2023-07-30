use core::arch::{asm, global_asm};

use bit_field::BitField;
use page_table::addr::VirtAddr;
use riscv::register::{sepc, sscratch, stval};
use riscv::register::sstatus::SPP;

pub use context::TrapFrame;
pub use exception::trap_common_read_file;
use syscall_define::signal::SIGNAL_RETURN_TRAP;
use syscall_define::signal::SignalNumber;
use syscall_define::time::TimerType;

use crate::arch::{external_interrupt_enable, hart_id, interrupt_disable, interrupt_enable, is_interrupt_enable, timer_interrupt_enable};
use crate::arch::riscv::register::scause::{Exception, Interrupt, Trap};
use crate::arch::riscv::register::stvec;
use crate::arch::riscv::register::stvec::TrapMode;
use crate::arch::riscv::sstatus;
use crate::config::TRAMPOLINE;
use crate::error::AlienError;
use crate::ipc::{send_signal, signal_handler, signal_return, solve_futex_wait};
use crate::memory::KERNEL_SPACE;
use crate::task::{current_task, current_trap_frame, current_user_token, do_exit, do_suspend};
use crate::timer::{check_timer_queue, set_next_trigger, set_next_trigger_in_kernel};

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
    check_timer_interrupt_pending();
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

#[inline]
fn set_user_trap_entry() {
    unsafe {
        stvec::write(TRAMPOLINE, TrapMode::Direct);
    }
}

#[inline]
pub fn set_kernel_trap_entry() {
    unsafe {
        sscratch::write(kernel_trap_vector as usize);
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
    fn do_kernel_handle(&self);
}

impl TrapHandler for Trap {
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
                    if err == AlienError::ThreadNeedWait {
                        // println!("thread need wait");
                        do_suspend();
                    } else if err == AlienError::ThreadNeedExit {
                        do_exit(-1);
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
                interrupt::external_interrupt_handler();
            }
            _ => {
                panic!(
                    "unhandled trap: {:?}, stval: {:?}, sepc: {:x}",
                    self, stval, sepc
                );
            }
        }
    }
    fn do_kernel_handle(&self) {
        let stval = stval::read();
        let sepc = sepc::read();
        match self {
            Trap::Interrupt(Interrupt::SupervisorTimer) => {
                trace!("[kernel] timer interrupt");
                check_timer_queue();
                solve_futex_wait();
                // set_next_trigger();
                set_next_trigger_in_kernel();
            }
            Trap::Exception(Exception::StorePageFault) => {
                debug!(
                    "[kernel] {:?} in kernel, stval:{:#x?} sepc:{:#x?}",
                    self, stval, sepc
                );
                {
                    let kernel_space = KERNEL_SPACE.read();
                    let phy = kernel_space.query(VirtAddr::from(stval));
                    debug!("physical address: {:#x?}", phy);
                }
            }
            Trap::Exception(_) => {
                panic!(
                    "unhandled trap: {:?}, stval: {:#x?}, sepc: {:#x}",
                    self, stval, sepc
                )
            }
            Trap::Interrupt(Interrupt::SupervisorExternal) => {
                interrupt::external_interrupt_handler();
            }
            _ => {
                panic!(
                    "unhandled trap: {:?}, stval: {:?}, sepc: {:x}",
                    self, stval, sepc
                )
            }
        }
        // check timer interrupt
        check_timer_interrupt_pending();
    }
}

pub fn check_timer_interrupt_pending() {
    let sip = riscv::register::sip::read();
    if sip.stimer() {
        debug!("timer interrupt pending");
        set_next_trigger();
    }
    let sie = riscv::register::sie::read();
    if !sie.stimer() {
        panic!("[check_timer_interrupt_pending] timer interrupt not enable");
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
    // update process statistics
    {
        let task = current_task().unwrap_or_else(|| {
            panic!(
                "can't find task in hart {}, but it's in user mode",
                hart_id() as usize
            )
        });
        task.access_inner().update_user_mode_time();
        check_task_timer_expired();
    }
    set_kernel_trap_entry();
    let cause = riscv::register::scause::read();
    let cause = cause.cause();
    cause.do_user_handle();
    let process = current_task().unwrap();
    if cause != Trap::Interrupt(Interrupt::SupervisorTimer) {
        // update process statistics
        process.access_inner().update_kernel_mode_time();
    }
    process.access_inner().update_timer();
    check_task_timer_expired();
    trap_return();
}

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
pub fn kernel_trap_vector() {
    let sstatus = sstatus::read();
    let spp = sstatus.spp();
    if spp == SPP::User {
        panic!("kernel_trap_vector: spp == SPP::User");
    }
    let enable = is_interrupt_enable();
    assert!(!enable);
    let cause = riscv::register::scause::read().cause();
    cause.do_kernel_handle()
}
