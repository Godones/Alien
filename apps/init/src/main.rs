#![no_std]
#![no_main]

extern crate alloc;

use Mstd::process::{exec, exit, fork, waitpid};
use Mstd::system_shutdown;
use Mstd::thread::m_yield;

#[no_mangle]
fn main() -> isize {
    // if fork() == 0 {
    //     exec("/bin/shell\0", &[0 as *const u8], &[0 as *const u8]);
    // } else {
    //     loop {
    //         let mut exit_code: i32 = 0;
    //         let tid = wait(&mut exit_code);
    //         if tid == -1 {
    //             m_yield();
    //             continue;
    //         }
    //         println!(
    //             "[Init] Released a task, tid={}, exit_code={}",
    //             tid, exit_code,
    //         );
    //     }
    // }
    // 0
    run_test();
    system_shutdown();
}

const ENV: &[*const u8] = &[
    "SHELL=/bash\0".as_ptr(),
    "LOGNAME=root\0".as_ptr(),
    "HOME=/root\0".as_ptr(),
    "USER=root\0".as_ptr(),
    "SHLVL=0\0".as_ptr(),
    "OLDPWD=/root\0".as_ptr(),
    "PS1=\x1b[1m\x1b[32mAlien\x1b[0m:\x1b[1m\x1b[34m\\w\x1b[0m\\$ \0".as_ptr(),
    "_=/bin/bash\0".as_ptr(),
    "PATH=/:/bin\0".as_ptr(),
    "LD_LIBRARY_PATH=/\0".as_ptr(),
    core::ptr::null(),
];

fn run_test() {
    let commands = [
        "./time-test\0",
        "./run-static.sh\0",
        "./run-dynamic.sh\0",
        "./libc-bench2\0",
        "./lua_testcode.sh\0",
        "./busybox_testcode.sh\0",
        "./cyclictest_testcode.sh\0",
        "./iozone_testcode.sh\0",
        "./unixbench_testcode.sh\0",
        "./lmbench_testcode.sh\0",
    ];
    commands.into_iter().for_each(|app| {
        let args = [app.as_ptr()];
        let pid = fork();
        if pid == 0 {
            exec(app, &args, ENV);
            exit(0);
        } else {
            m_yield();
            let mut exit_code: i32 = 0;
            let _x = waitpid(pid as usize, &mut exit_code);
        }
    });
}
