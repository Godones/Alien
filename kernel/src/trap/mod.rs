mod exception;

use alloc::sync::Arc;
use core::arch::{asm, global_asm};

use arch::hart_id;
use basic::sync::{Once, OnceGet};
use config::TRAMPOLINE;
use interface::{PLICDomain, SysCallDomain};
use log::debug;
use platform::println;
use riscv::register::{
    scause::{Exception, Interrupt, Trap},
    sepc, sscratch, sstatus,
    sstatus::SPP,
    stval, stvec,
    stvec::TrapMode,
};

use crate::{task_domain, timer};

pub static SYSCALL_DOMAIN: Once<Arc<dyn SysCallDomain>> = Once::new();
pub static PLIC_DOMAIN: Once<Arc<dyn PLICDomain>> = Once::new();

#[macro_export]
macro_rules! syscall_domain {
    () => {
        $crate::trap::SYSCALL_DOMAIN.get_must()
    };
}

macro_rules! plic_domain {
    () => {
        crate::trap::PLIC_DOMAIN.get_must()
    };
}

pub fn register_syscall_domain(syscall_domain: Arc<dyn SysCallDomain>) {
    SYSCALL_DOMAIN.call_once(|| syscall_domain);
}

pub fn register_plic_domain(plic_domain: Arc<dyn PLICDomain>) {
    PLIC_DOMAIN.call_once(|| plic_domain);
}

global_asm!(include_str!("./kernel_v.asm"));
global_asm!(include_str!("./trampoline.asm"));

extern "C" {
    fn kernel_v();
    fn user_v();
    fn user_r();
}

/// 开启中断/异常
pub fn init_trap_subsystem() {
    println!("++++ setup interrupt ++++");
    set_kernel_trap_entry();
    arch::external_interrupt_enable();
    arch::timer_interrupt_enable();
    arch::interrupt_enable();
    let enable = arch::is_interrupt_enable();
    println!("++++ setup interrupt done, enable:{:?} ++++", enable);
}
/// 设置内核态 trap 处理例程的入口点
#[inline]
pub fn set_kernel_trap_entry() {
    unsafe {
        sscratch::write(kernel_trap_vector as usize);
        stvec::write(kernel_v as usize, TrapMode::Direct);
    }
}

/// 设置用户态 trap 处理例程的入口点
#[inline]
fn set_user_trap_entry() {
    unsafe {
        stvec::write(TRAMPOLINE, TrapMode::Direct);
    }
}

/// 只有在内核态下才能进入这个函数
/// 避免嵌套中断发生这里不会再开启中断
#[no_mangle]
pub fn kernel_trap_vector(sp: usize) {
    let sstatus = sstatus::read();
    let spp = sstatus.spp();
    if spp == SPP::User {
        panic!("kernel_trap_vector: spp == SPP::User");
    }
    let enable = arch::is_interrupt_enable();
    assert!(!enable);
    let cause = riscv::register::scause::read().cause();
    cause.do_kernel_handle(sp)
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
        // let task = current_task().expect("user_trap_vector: current_task is none");
        // update process statistics
        // task.access_inner().update_user_mode_time();
        // check_task_timer_expired();
        // }
        set_kernel_trap_entry();
        let cause = riscv::register::scause::read();
        let cause = cause.cause();
        cause.do_user_handle();
        // let task = current_task().unwrap();
        // if cause != Trap::Interrupt(Interrupt::SupervisorTimer) {
        //     // update process statistics
        //     task.access_inner().update_kernel_mode_time();
        // }
        // task.access_inner().update_timer();
        // check_task_timer_expired();
        trap_return();
    }
}
#[no_mangle]
pub fn trap_return() -> ! {
    // signal_handler();
    // arch::interrupt_disable();
    set_user_trap_entry();
    let task_domain = task_domain!();
    let (user_satp, trap_cx_ptr) = task_domain.satp_with_trap_frame_virt_addr().unwrap();
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

pub trait TrapHandler {
    fn do_user_handle(&self);
    fn do_kernel_handle(&self, sp: usize);
}

impl TrapHandler for Trap {
    /// 用户态下的 trap 例程
    fn do_user_handle(&self) {
        let stval = stval::read();
        let sepc = sepc::read();
        debug!("trap :{:?}", self);
        match self {
            Trap::Exception(Exception::UserEnvCall) => {
                exception::syscall_exception_handler();
            }
            Trap::Exception(Exception::StoreFault)
            | Trap::Exception(Exception::LoadFault)
            | Trap::Exception(Exception::InstructionFault)
            | Trap::Exception(Exception::IllegalInstruction) => {
                panic!(
                    "<do_user_handle> {:?} in application,stval:{:#x?} sepc:{:#x?}",
                    self, stval, sepc
                );
            }
            Trap::Exception(Exception::StorePageFault)
            | Trap::Exception(Exception::LoadPageFault) => {
                task_domain!()
                    .do_load_page_fault(stval)
                    .expect("do_load_page_fault failed");
                debug!(
                    "<do_user_handle> {:?}, stval:{:#x?} sepc:{:#x?}",
                    self, stval, sepc
                );
            }
            Trap::Exception(Exception::InstructionPageFault) => {
                panic!("<do_user_handle> instruction page fault")
            }
            Trap::Interrupt(Interrupt::SupervisorTimer) => {
                trace!("<do_user_handle> timer interrupt");
                timer::set_next_trigger();
                crate::task::yield_now();
            }
            Trap::Interrupt(Interrupt::SupervisorExternal) => {
                trace!("[{}] <do_user_handle> external interrupt", hart_id());
                plic_domain!().handle_irq().expect("handle_irq failed");
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
    fn do_kernel_handle(&self, _sp: usize) {
        let stval = stval::read();
        let sepc = sepc::read();
        match self {
            Trap::Interrupt(Interrupt::SupervisorTimer) => {
                trace!("<do_kernel_handle> timer interrupt");
                // record_irq(1);
                // check_timer_queue();
                // solve_futex_wait();
                timer::set_next_trigger()
            }
            Trap::Exception(_) => {
                panic!(
                    "[kernel] {:?} in kernel, stval:{:#x?} sepc:{:#x?}",
                    self, stval, sepc
                );
            }
            Trap::Interrupt(Interrupt::SupervisorExternal) => {
                println!("<do_kernel_handle> external interrupt");
                plic_domain!().handle_irq().expect("handle_irq failed");
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
