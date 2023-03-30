use crate::syscall::sys_get_time;

pub fn get_time_ms() -> isize {
    sys_get_time()
}
