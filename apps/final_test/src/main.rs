#![no_main]
#![no_std]

use Mstd::process::{exec, exit, fork, waitpid};
use Mstd::thread::m_yield;
use Mstd::{println, system_shutdown};

#[no_mangle]
fn main() -> isize {
    run_test();
    println!("!TEST FINISH!");
    system_shutdown();
}
fn run_test() {
    let commands = [
        "./time-test\0",
        "./interrupts-test-1\0",
        "./interrupts-test-2\0",
        "./copy-file-range-test-1\0",
        "./copy-file-range-test-2\0",
        "./copy-file-range-test-3\0",
        "./copy-file-range-test-4\0",
        // "./lua_testcode.sh\0",
        // "./busybox_testcode.sh\0",
        // "./run-static.sh\0",
        // "./run-dynamic.sh\0",
        // "./libc-bench\0",
        // "./cyclictest_testcode.sh\0",
        // "./netperf_testcode.sh\0",
        // "./iperf_testcode.sh\0",
        // "./lmbench_testcode.sh\0",
        // "./iozone_testcode.sh\0",
        // "./unixbench_testcode.sh\0",
    ];
    commands.into_iter().for_each(|app| {
        let args = [app.as_ptr()];
        let pid = fork();
        if pid == 0 {
            exec(app, &args, BASH_ENV);
            exit(0);
        } else {
            m_yield();
            let mut exit_code: i32 = 0;
            let _x = waitpid(pid as usize, &mut exit_code);
        }
    });
}
const BASH_ENV: &[*const u8] = &[
    "SHELL=/bash\0".as_ptr(),
    "PWD=/\0".as_ptr(),
    "LOGNAME=root\0".as_ptr(),
    "MOTD_SHOWN=pam\0".as_ptr(),
    "HOME=/root\0".as_ptr(),
    "LANG=C.UTF-8\0".as_ptr(),
    "TERM=vt220\0".as_ptr(),
    "USER=root\0".as_ptr(),
    "SHLVL=0\0".as_ptr(),
    "OLDPWD=/root\0".as_ptr(),
    "PS1=\x1b[1m\x1b[32mAlien\x1b[0m:\x1b[1m\x1b[34m\\w\x1b[0m\\$ \0".as_ptr(),
    "_=/bin/bash\0".as_ptr(),
    "PATH=/:/bin\0".as_ptr(),
    "LD_LIBRARY_PATH=/bin\0".as_ptr(),
    core::ptr::null(),
];
