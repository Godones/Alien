use config::CLOCK_FREQ;
use pconst::time::TimeSpec;

pub fn read_time_ms() -> u64 {
    get_time_ms()
}

#[inline]
fn read_timer() -> usize {
    arch::read_timer()
}

#[inline]
fn get_time_ms() -> u64 {
    const MSEC_PER_SEC: usize = 1000;
    (read_timer() / (CLOCK_FREQ / MSEC_PER_SEC)) as u64
}

pub trait ToClock {
    fn to_clock(&self) -> usize;
}

pub trait TimeNow {
    fn now() -> Self;
}

impl ToClock for TimeSpec {
    fn to_clock(&self) -> usize {
        self.tv_sec * CLOCK_FREQ + self.tv_nsec * CLOCK_FREQ / 1000_000_000
    }
}

impl TimeNow for TimeSpec {
    fn now() -> Self {
        let time = read_timer();
        Self {
            tv_sec: time / CLOCK_FREQ,
            tv_nsec: (time % CLOCK_FREQ) * 1000000000 / CLOCK_FREQ,
        }
    }
}
