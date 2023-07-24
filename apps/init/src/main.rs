#![no_std]
#![no_main]

extern crate alloc;

use Mstd::process::{exec, exit, fork, wait, waitpid};
use Mstd::shutdown;
use Mstd::thread::m_yield;

#[no_mangle]
fn main() -> isize {
    if fork() == 0 {
        // exec("/bin/shell\0", &[0 as *const u8], &[0 as *const u8]);
        run_test();
    } else {
        loop {
            let mut exit_code: i32 = 0;
            let tid = wait(&mut exit_code);
            if tid == -1 {
                m_yield();
                continue;
            }
            // println!(
            //     "[Init] Released a task, tid={}, exit_code={}",
            //     tid, exit_code,
            // );
            shutdown();
        }
    }
    0
}

fn run_test() {
    let commands = [
        "./time-test\0",
        "./busybox_testcode.sh\0",
        "./lua_testcode.sh\0",
        "./iozone_testcode.sh\0",
    ];
    commands.into_iter().for_each(|app| {
        let args = [app.as_ptr()];
        let pid = fork();
        if pid == 0 {
            exec(app, &args, &[0 as *const u8]);
            exit(0);
        } else {
            m_yield();
            let mut exit_code: i32 = 0;
            let _x = waitpid(pid as usize, &mut exit_code);
        }
    });
}
