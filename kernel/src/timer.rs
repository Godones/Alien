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

pub struct TimeTick {
    start: usize,
    info: &'static str,
}

impl TimeTick {
    pub fn new(info: &'static str) -> Self {
        TimeTick {
            info,
            start: read_timer(),
        }
    }
}

impl Drop for TimeTick {
    fn drop(&mut self) {
        let end = read_timer();
        println_color!(
            35,
            "[{}] Time elapsed: {} us",
            self.info,
            (end - self.start) * 1000_000 / CLOCK_FREQ
        );
    }
}
