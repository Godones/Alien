use alloc::{format, vec::Vec};

use Mstd::{
    process::{getpid, CloneFlags},
    thread::{gettid, thread_create},
    time::sleep,
};

const STACK_SIZE: usize = 2 * 1024;

fn thread_fun() {
    println!("This is child thread!");
    print_id();
}

pub fn thread_test1() -> isize {
    let ustack = [0u8; STACK_SIZE];
    let sp: *const u8 = &ustack as *const [u8; STACK_SIZE] as *const u8;
    // CloneFlags::CLONE_THREAD
    let flags: u32 = 0x00010000;
    let args = Vec::new();
    unsafe {
        let tid = thread_create(
            thread_fun as *const u32,
            (sp as usize + STACK_SIZE) as *const u32,
            flags,
            &args as &[*const u8],
        );
        assert!(tid != 0);
    }
    print_id();
    sleep(500);
    // pthread_join(tid, NULL);
    0
}

fn print_id() {
    println!("pid: {}, tid: {}", getpid(), gettid());
}

// fn thread_fun1(){
//     println!("thread 1 return \n");

// }

// fn thread_fun2() -> !{
//     println!("thread2 exit");
//     pthread_exit((void *)2);
// }
