#![no_main]
#![no_std]

extern crate alloc;

use alloc::string::ToString;

use Mstd::println;
use Mstd::process::{exec, exit, fork, waitpid};
use Mstd::thread::m_yield;

#[no_mangle]
fn main() -> isize {
    let commands = [
        "brk",
        "chdir",
        "clone",
        "close",
        "dup",
        "dup2",
        "execve",
        "exit",
        "fork",
        "fstat",
        "getcwd",
        "getpid",
        "getppid",
        "gettimeofday",
        "mkdir_",
        "mmmap",
        "mount",
        "munmap",
        "open",
        "times",
        "openat",
        "pipe",
        "read",
        "sleep",
        "umount",
        "uname",
        "unlink",
        "wait",
        "waitpid",
        "getdents",
        "write",
        "yield",
    ];

    commands.into_iter().for_each(|app| {
        println!("run {}", app);
        let mut app = app.to_string();
        app.push('\0');
        let app = "/ostest/".to_string() + app.as_str();
        let pid = fork();
        if pid == 0 {
            exec(app.as_str(), &[0 as *const u8]);
            exit(0);
        } else {
            m_yield();
            let mut exit_code: i32 = 0;
            let x = waitpid(pid as usize, &mut exit_code);
            println!("waitpid: {}, exit_code: {}", x, exit_code);
        }
    });
    0
}
