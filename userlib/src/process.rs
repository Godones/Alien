use crate::syscall::{sys_execve, sys_exit, sys_fork, sys_getpid, sys_waitpid};
use crate::thread::m_yield;
use alloc::string::ToString;

pub fn exit(exit_code: i32) -> ! {
    sys_exit(exit_code);
    loop {}
}

pub fn fork() -> isize {
    sys_fork()
}
pub fn getpid() -> isize {
    sys_getpid()
}

pub fn exec(cmd: &str, args: &[*const u8]) -> isize {
    sys_execve(cmd.as_ptr(), args.as_ptr() as *const usize)
}

pub fn wait(exit_code: &mut i32) -> isize {
    sys_waitpid(-1, exit_code as *mut _)
}

pub fn waitpid(pid: usize, exit_code: &mut i32) -> isize {
    sys_waitpid(pid as isize, exit_code as *mut _)
}
