use crate::syscall::{sys_execve, sys_exit, sys_fork, sys_getpid, sys_waitpid};
use crate::thread::m_yield;

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

pub fn exec(path: &str) -> isize {
    sys_execve(path.as_ptr())
}

pub fn wait(exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(-1, exit_code as *mut _) {
            -2 => {
                m_yield();
            }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}

pub fn waitpid(pid: usize, exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(pid as isize, exit_code as *mut _) {
            -2 => {
                m_yield();
            }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}
