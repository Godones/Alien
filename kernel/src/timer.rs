use platform::config::CLOCK_FREQ;

/// 获取当前计时器的值
#[inline]
pub fn read_timer() -> usize {
    arch::read_timer()
}

/// 获取当前时间，以 ms 为单位
pub fn get_time_ms() -> isize {
    const MSEC_PER_SEC: usize = 1000;
    (read_timer() / (CLOCK_FREQ / MSEC_PER_SEC)) as isize
}

#[inline]
pub fn set_next_trigger() {
    const TICKS_PER_SEC: usize = 10;
    let next = read_timer() + CLOCK_FREQ / TICKS_PER_SEC;
    assert!(next > read_timer());
    platform::set_timer(next);
}
