use crate::syscall::{sys_gettid, sys_yield};

pub fn m_yield() -> isize {
    sys_yield()
}

pub fn gettid() -> isize {
    sys_gettid()
}

pub fn thread_create(fnc: *const u32, ustack: *const u32, flags: u32, args: &[*const u8]) -> isize {
    // sys_clone(
    //     fnc as *const usize,
    //     ustack as *const usize,
    //     flags,
    //     args.as_ptr() as *const usize,
    // )
    0
}
