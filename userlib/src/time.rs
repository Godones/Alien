use crate::syscall::sys_get_time;

pub fn get_time() -> isize {
    sys_get_time()
}
