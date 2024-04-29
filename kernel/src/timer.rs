use platform::config::CLOCK_FREQ;

/// 获取当前计时器的值
#[inline]
pub fn read_timer() -> usize {
    arch::read_timer()
}

#[inline]
pub fn set_next_trigger() {
    const TICKS_PER_SEC: usize = 10;
    let next = read_timer() + CLOCK_FREQ / TICKS_PER_SEC;
    assert!(next > read_timer());
    platform::set_timer(next);
}
