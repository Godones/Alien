use core::arch::{asm, global_asm};

use page_table::addr::VirtAddr;
use riscv::register::sstatus::SPP;
use riscv::register::{sepc, sscratch, stval};

pub use context::TrapFrame;
use syscall_define::signal::SignalNumber;
use syscall_define::time::TimerType;

use crate::arch::riscv::register::scause::{Exception, Interrupt, Trap};
use crate::arch::riscv::register::stvec;
use crate::arch::riscv::register::stvec::TrapMode;
use crate::arch::riscv::sstatus;
use crate::arch::{
    external_interrupt_enable, hart_id, interrupt_disable, interrupt_enable, is_interrupt_enable,
    timer_interrupt_enable,
};
use crate::config::TRAMPOLINE;
use crate::ipc::{send_signal, signal_handler, solve_futex_wait};
use crate::memory::KERNEL_SPACE;
use crate::task::{current_task, current_user_token};
use crate::timer::{check_timer_queue, set_next_trigger};

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
        stvec::write(TRAMPOLINE as usize, TrapMode::Direct);
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
    set_kernel_trap_entry();
    interrupt_enable();
    external_interrupt_enable();
    timer_interrupt_enable();
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
                    "[kernel] {:?} in application,stval:{:#x?} sepc:{:#x?}",
                    self, stval, sepc
                );
                let task = current_task().unwrap();
                send_signal(task.get_tid() as usize, SignalNumber::SIGSEGV as usize)
            }
            Trap::Exception(Exception::StorePageFault)
            | Trap::Exception(Exception::LoadPageFault) => {
                let res = exception::page_exception_handler(self.clone(), stval);
                if res.is_err() {
                    error!(
                        "[kernel] {:?} in application,stval:{:#x?} sepc:{:#x?}",
                        self, stval, sepc
                    );
                    let task = current_task().unwrap();
                    send_signal(task.get_tid() as usize, SignalNumber::SIGSEGV as usize)
                }
            }
            Trap::Exception(Exception::InstructionPageFault) => {
                // todo!("instruction page fault");
                error!(
                    "[kernel] {:?} in application,stval:{:#x?} sepc:{:#x?}",
                    self, stval, sepc
                );
                let task = current_task().unwrap();
                send_signal(task.get_tid() as usize, SignalNumber::SIGSEGV as usize)
            }
            Trap::Interrupt(Interrupt::SupervisorTimer) => {
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
                trace!("timer interrupt");
                check_timer_queue();
                solve_futex_wait();
                set_next_trigger();
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
    if cause != Trap::Interrupt(Interrupt::SupervisorTimer) {
        // update process statistics
        let process = current_task().unwrap();
        process.access_inner().update_kernel_mode_time();
        check_task_timer_expired();
    }
    trap_return();
}

fn check_task_timer_expired() {
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
