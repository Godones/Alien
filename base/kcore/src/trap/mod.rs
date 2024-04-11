mod exception;

use alloc::boxed::Box;
use core::{
    arch::{asm, global_asm},
    fmt::Debug,
};

use bit_field::BitField;
use config::TRAMPOLINE;
use context::TrapFrameRaw;
use platform::println;
use riscv::register::{
    scause::{Exception, Interrupt, Trap},
    sepc, sscratch, sstatus,
    sstatus::SPP,
    stval, stvec,
    stvec::TrapMode,
};

use crate::{PLIC_DOMAIN, TASK_DOMAIN};

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
    println!("{:?}", spp);
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
    // let sstatus = trap_frame.get_status();
    let sstatus = riscv::register::sstatus::read();
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
    arch::interrupt_disable();
    set_user_trap_entry();

    let task_domain = TASK_DOMAIN.get().unwrap();
    let trap_frame_ptr = task_domain.trap_frame_virt_addr().unwrap();

    // let sstatues = trap_frame.get_status();
    // let enable = sstatues.0.get_bit(5);
    // let sie = sstatues.0.get_bit(1);
    // assert!(enable);
    // assert!(!sie);
    let trap_cx_ptr = trap_frame_ptr as *const TrapFrameRaw;
    let user_satp = task_domain.current_task_satp().unwrap();
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
                // let task = current_task().unwrap();
                // send_signal(task.get_tid() as usize, SignalNumber::SIGSEGV as usize)
            }
            Trap::Exception(Exception::StorePageFault)
            | Trap::Exception(Exception::LoadPageFault) => {
                // let task = current_task().unwrap();
                // let tid = task.get_tid();
                // warn!(
                //     "[User][tid:{}] {:?} in application,stval:{:#x?} sepc:{:#x?}",
                //     tid, self, stval, sepc
                // );
                // let res = exception::page_exception_handler(self.clone(), stval);
                // if res.is_err() {
                //     error!(
                //         "[User] {:?} in application,stval:{:#x?} sepc:{:#x?}",
                //         self, stval, sepc
                //     );
                //     let err = res.err().unwrap();
                //     if err == AlienError::EAGAIN {
                //         // println!("thread need wait");
                //         do_suspend();
                //     } else if err == AlienError::ETMP {
                //         do_exit(-1);
                //     } else {
                //         send_signal(tid as usize, SignalNumber::SIGSEGV as usize)
                //     }
                // }
                panic!("page fault");
            }
            Trap::Exception(Exception::InstructionPageFault) => {
                // trace!(
                //     "[User] {:?} in application,stval:{:#x?} sepc:{:#x?}",
                //     self,
                //     stval,
                //     sepc
                // );
                // if stval == SIGNAL_RETURN_TRAP {
                //     // 当作调用了 sigreturn 一样
                //     let cx = current_trap_frame();
                //     cx.regs()[10] = signal_return() as usize;
                //     return;
                // }
                //
                // let res = exception::page_exception_handler(self.clone(), stval);
                // if res.is_err() {
                //     error!(
                //         "[User] {:?} in application,stval:{:#x?} sepc:{:#x?}",
                //         self, stval, sepc
                //     );
                //     let task = current_task().unwrap();
                //     send_signal(task.get_tid() as usize, SignalNumber::SIGSEGV as usize)
                // }
            }
            Trap::Interrupt(Interrupt::SupervisorTimer) => {
                trace!("[User] timer interrupt");
                timer::set_next_trigger();
                TASK_DOMAIN.get().unwrap().do_yield();
            }
            Trap::Interrupt(Interrupt::SupervisorExternal) => {
                trace!("[User] external interrupt");
                PLIC_DOMAIN.get().unwrap().handle_irq();
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
    fn do_kernel_handle(&self, sp: usize) {
        let stval = stval::read();
        let sepc = sepc::read();
        match self {
            Trap::Interrupt(Interrupt::SupervisorTimer) => {
                debug!("[kernel] timer interrupt");
                // record_irq(1);
                // check_timer_queue();
                // solve_futex_wait();
                // set_next_trigger_in_kernel();
                timer::set_next_trigger_in_kernel()
            }
            Trap::Exception(Exception::StorePageFault) => {
                debug!(
                    "[kernel] {:?} in kernel, stval:{:#x?} sepc:{:#x?}",
                    self, stval, sepc
                );
                // {
                //     let phy = mem::query_kernel_space(stval);
                //     debug!("physical address: {:#x?}", phy);
                // }
            }
            Trap::Exception(_) => {
                panic!(
                    "unhandled trap: {:?}, stval: {:#x?}, sepc: {:#x}, sp: {:#x}",
                    self, stval, sepc, 0
                )
            }
            Trap::Interrupt(Interrupt::SupervisorExternal) => {
                PLIC_DOMAIN.get().unwrap().handle_irq();
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
