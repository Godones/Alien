use bitflags::bitflags;
use pconst::task::WaitOptions;

use crate::syscall::{sys_execve, sys_exit, sys_fork, sys_getpid, sys_waitpid};

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

pub fn exec(cmd: &str, args: &[*const u8], env: &[*const u8]) -> isize {
    sys_execve(
        cmd.as_ptr(),
        args.as_ptr() as *const usize,
        env.as_ptr() as *const usize,
    )
}

pub fn wait(exit_code: &mut i32, option: WaitOptions) -> isize {
    sys_waitpid(-1, exit_code as *mut _, option.bits())
}

pub fn waitpid(pid: usize, exit_code: &mut i32) -> isize {
    sys_waitpid(pid as isize, exit_code as *mut _, 0)
}

bitflags! {
    pub struct CloneFlags: u32 {
        const CLONE_VM = 0x00000100;
        const CLONE_FS = 0x00000200;
        const CLONE_FILES = 0x00000400;
        const CLONE_SIGHAND = 0x00000800;
        const CLONE_PTRACE = 0x00002000;
        const CLONE_VFORK = 0x00004000;
        const CLONE_PARENT = 0x00008000;
        const CLONE_THREAD = 0x00010000;
        const CLONE_NEWNS = 0x00020000;
        const CLONE_SYSVSEM = 0x00040000;
        const CLONE_SETTLS = 0x00080000;
        const CLONE_PARENT_SETTID = 0x00100000;
        const CLONE_CHILD_CLEARTID = 0x00200000;
        const CLONE_DETACHED = 0x00400000;
        const CLONE_UNTRACED = 0x00800000;
        const CLONE_CHILD_SETTID = 0x01000000;
        const CLONE_NEWCGROUP = 0x02000000;
        const CLONE_NEWUTS = 0x04000000;
        const CLONE_NEWIPC = 0x08000000;
        const CLONE_NEWUSER = 0x10000000;
        const CLONE_NEWPID = 0x20000000;
        const CLONE_NEWNET = 0x40000000;
        const CLONE_IO = 0x80000000;
    }
}

bitflags! {
    pub struct SignalFlags:u32 {
        const SIGHUP = 1;
        const SIGINT = 2;
        const SIGQUIT = 3;
        const SIGILL = 4;
        const SIGTRAP = 5;
        const SIGABRT = 6;
        const SIGBUS = 7;
        const SIGFPE = 8;
        const SIGKILL = 9;
        const SIGUSR1 = 10;
        const SIGSEGV = 11;
        const SIGUSR2 = 12;
        const SIGPIPE = 13;
        const SIGALRM = 14;
        const SIGTERM = 15;
        const SIGSTKFLT = 16;
        const SIGCHLD = 17;
        const SIGCONT = 18;
        const SIGSTOP = 19;
        const SIGTSTP = 20;
        const SIGTTIN = 21;
        const SIGTTOU = 22;
        const SIGURG = 23;
        const SIGXCPU = 24;
        const SIGXFSZ = 25;
        const SIGVTALRM = 26;
        const SIGPROF = 27;
        const SIGWINCH = 28;
        const SIGIO = 29;
        const SIGPWR = 30;
        const SIGSYS = 31;
    }
}
