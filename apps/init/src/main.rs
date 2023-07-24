#![no_std]
#![no_main]

use Mstd::shutdown;
use Mstd::process::{exec, fork, wait};
use Mstd::thread::m_yield;

#[no_mangle]
fn main() -> isize {
    if fork() == 0 {
        // exec("/bin/shell\0", &[0 as *const u8], &[0 as *const u8]);
        run_test("./busybox_testcode.sh\0");
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


fn run_test(sh: &str) {
    let args = &[sh.as_ptr()];
    exec(sh, args, &[0 as *const u8]);
}