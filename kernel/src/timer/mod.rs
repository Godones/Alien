use syscall_table::syscall_func;
use crate::arch;
use crate::config::CLOCK_FREQ;

/// 获取当前计时器的值
#[inline]
pub fn read_timer() -> usize {
    arch::read_timer()
}

/// 设置下一次时钟的中断
#[inline]
pub fn set_next_trigger(addition: usize) {
    crate::sbi::set_timer(read_timer() + addition)
}

const TICKS_PER_SEC: usize = 100;
const MSEC_PER_SEC: usize = 1000;

#[syscall_func(169)]
pub fn get_time_ms() -> isize {
    (read_timer() / (CLOCK_FREQ / MSEC_PER_SEC)) as isize
}
