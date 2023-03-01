use crate::syscall::{SysCallID, Syscall};
use crate::task::current_trap_frame;

pub fn syscall_exception_handler() {
    // jump to next instruction anyway
    let mut cx = current_trap_frame();
    cx.update_sepc();
    // get system call return value
    let syscall = SysCallID::from(cx.parameters());
    let result = syscall.do_syscall();
    // cx is changed during sys_exec, so we have to call it again
    cx = current_trap_frame();
    cx.update_res(result as usize);
}

/// the solution for page fault
pub fn page_exception_handler() {
    let syscall = SysCallID::Exit(-2);
    syscall.do_syscall();
}

/// the solution for illegal instruction
pub fn illegal_instruction_exception_handler() {
    let syscall = SysCallID::Exit(-3);
    syscall.do_syscall();
}
