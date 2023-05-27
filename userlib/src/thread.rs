use crate::syscall::sys_yield;

pub fn m_yield() -> isize {
    sys_yield()
}
