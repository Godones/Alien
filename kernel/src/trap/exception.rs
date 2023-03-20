use crate::syscall;
use crate::task::current_trap_frame;

pub fn syscall_exception_handler() {
    // jump to next instruction anyway
    let mut cx = current_trap_frame();
    cx.update_sepc();
    // get system call return value
    let parameters = cx.parameters();
    let result = syscall::do_syscall(parameters[0], &parameters[1..]);
    // cx is changed during sys_exec, so we have to call it again
    cx = current_trap_frame();
    cx.update_res(result as usize);
}

/// the solution for page fault
pub fn page_exception_handler() {
    let args = [-1isize as usize];
    syscall::do_syscall(93, &args);
}

/// the solution for illegal instruction
pub fn illegal_instruction_exception_handler() {
    let args = [-3isize as usize];
    syscall::do_syscall(93, &args);
}
