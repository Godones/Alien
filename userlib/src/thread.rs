use crate::syscall::{sys_get_time, sys_yield};

pub fn m_yield() -> isize {
    sys_yield()
}

pub fn sleep(period_ms: usize) {
    let start = sys_get_time();
    while sys_get_time() < start + period_ms as isize {
        sys_yield();
    }
}
