use crate::syscall::{sys_sleep, sys_yield};

pub fn m_yield() -> isize {
    sys_yield()
}

pub fn sleep(period_ms: usize) -> isize {
    sys_sleep(period_ms)
}
