use core::arch::{asm, global_asm};

use riscv::register::{sepc, sscratch, stval};
use riscv::register::sstatus::SPP;

pub use context::TrapFrame;

use crate::arch::{
    external_interrupt_enable, interrupt_disable, interrupt_enable, is_interrupt_enable,
    timer_interrupt_enable,
};
use crate::arch::riscv::register::scause::{Exception, Interrupt, Trap};
use crate::arch::riscv::register::stvec;
use crate::arch::riscv::register::stvec::TrapMode;
use crate::arch::riscv::sstatus;
use crate::config::{TRAMPOLINE, TRAP_CONTEXT_BASE};
use crate::memory::KERNEL_SPACE;
use crate::task::{current_process, current_user_token};
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
    interrupt_disable();
    set_user_trap_entry();
    let trap_cx_ptr = TRAP_CONTEXT_BASE;
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
fn set_kernel_trap_entry() {
    unsafe {
        sscratch::write(kernel_trap_vector as usize);
        stvec::write(kernel_v as usize, TrapMode::Direct);
    }
}

/// 开启中断/异常
pub fn init_trap_subsystem() {
    println!("kernel_v:{:x}", kernel_v as usize);
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
        match self {
            Trap::Exception(Exception::UserEnvCall) => {
                exception::syscall_exception_handler();
            }
            Trap::Exception(Exception::StoreFault)
            | Trap::Exception(Exception::StorePageFault)
            | Trap::Exception(Exception::InstructionFault)
            | Trap::Exception(Exception::InstructionPageFault)
            | Trap::Exception(Exception::LoadFault)
            | Trap::Exception(Exception::LoadPageFault) => {
                error!(
                    "[kernel] {:?} in application,stval:{:#x?} sepc:{:#x?}",
                    self, stval, sepc
                );
                exception::page_exception_handler(self.clone(), stval)
            }
            Trap::Interrupt(Interrupt::SupervisorTimer) => {
                interrupt::timer_interrupt_handler();
            }
            Trap::Exception(Exception::IllegalInstruction) => {
                error!("[kernel] IllegalInstruction in application, kernel killed it.");
                exception::illegal_instruction_exception_handler()
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
                check_timer_queue();
                set_next_trigger();
            }
            Trap::Exception(Exception::StorePageFault) => {
                debug!(
                    "[kernel] {:?} in kernel, stval:{:#x?} sepc:{:#x?}",
                    self, stval, sepc
                );
                {
                    let kernel_space = KERNEL_SPACE.read();
                    let phy = kernel_space.virtual_to_physical(stval);
                    debug!("physical address: {:#x?}", phy);
                }
            }
            Trap::Exception(_) => {
                panic!(
                    "unhandled trap: {:?}, stval: {:?}, sepc: {:x}",
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
    // update process statistics
    {
        let process = current_process().unwrap();
        process.access_inner().update_user_mode_time();
    }
    let sstatus = sstatus::read();
    let spp = sstatus.spp();
    if spp == SPP::Supervisor {
        panic!("user_trap_vector: spp == SPP::Supervisor");
    }
    set_kernel_trap_entry();
    let cause = riscv::register::scause::read();
    let cause = cause.cause();
    cause.do_user_handle();
    if cause != Trap::Interrupt(Interrupt::SupervisorTimer) {
        // update process statistics
        let process = current_process().unwrap();
        process.access_inner().update_kernel_mode_time();
    }
    trap_return();
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
