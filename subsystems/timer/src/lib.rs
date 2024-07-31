#![no_std]

use constants::time::{TimeSpec, TimeVal};
use platform::config::CLOCK_FREQ;
use vfscore::utils::VfsTimeSpec;
/// 每秒包含的毫秒数
const MSEC_PER_SEC: usize = 1000;
/// 程序运行时间
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Times {
    /// the ticks of user mode
    pub tms_utime: usize,
    /// the ticks of kernel mode
    pub tms_stime: usize,
    /// the ticks of user mode of child process
    pub tms_cutime: usize,
    /// the ticks of kernel mode of child process
    pub tms_cstime: usize,
}

impl Times {
    pub fn new() -> Self {
        Self {
            tms_utime: 0,
            tms_stime: 0,
            tms_cutime: 0,
            tms_cstime: 0,
        }
    }
}

/// 实现 `TimeNow` 特征的时钟结构，能够通过调用 `now` 方法得出 表示当前的 cpu 时间的一个本类型时钟
pub trait TimeNow {
    fn now() -> Self;
}

/// 实现 `ToClock` 特征的时钟结构，能够将所表示的时间间隔，转换为 cpu 时钟
pub trait ToClock {
    fn to_clock(&self) -> usize;
}

/// 实现 `TimeFromFreq` 特征的时钟结构，能够实现从 cpu时钟跳变的次数 初始化一个本类型的时钟
pub trait TimeFromFreq {
    fn from_freq(freq: usize) -> Self;
}

impl TimeNow for TimeVal {
    fn now() -> Self {
        let time = read_timer();
        Self {
            tv_sec: time / CLOCK_FREQ,
            tv_usec: (time % CLOCK_FREQ) * 1000000 / CLOCK_FREQ,
        }
    }
}
impl ToClock for TimeVal {
    fn to_clock(&self) -> usize {
        self.tv_sec * CLOCK_FREQ + self.tv_usec * CLOCK_FREQ / 1000_000
    }
}

impl TimeFromFreq for TimeVal {
    fn from_freq(freq: usize) -> Self {
        Self {
            tv_sec: freq / CLOCK_FREQ,
            tv_usec: (freq % CLOCK_FREQ) * 1000000 / CLOCK_FREQ,
        }
    }
}

// /// 更精细的时间，秒(s)+纳秒(ns)
// #[repr(C)]
// #[derive(Copy, Clone, Debug, PartialEq, Eq)]
// pub struct TimeSpec {
//     pub tv_sec: usize,
//     pub tv_nsec: usize, //0~999999999
// }
//

impl TimeNow for TimeSpec {
    fn now() -> Self {
        let time = read_timer();
        Self {
            tv_sec: time / CLOCK_FREQ,
            tv_nsec: (time % CLOCK_FREQ) * 1000_000_000 / CLOCK_FREQ,
        }
    }
}

impl TimeFromFreq for TimeSpec {
    fn from_freq(freq: usize) -> Self {
        Self {
            tv_sec: freq / CLOCK_FREQ,
            tv_nsec: (freq % CLOCK_FREQ) * 1000_000_000 / CLOCK_FREQ,
        }
    }
}

impl ToClock for TimeSpec {
    fn to_clock(&self) -> usize {
        self.tv_sec * CLOCK_FREQ + self.tv_nsec * CLOCK_FREQ / 1000_000_000
    }
}

pub trait ToVfsTimeSpec {
    fn into_vfs(self) -> VfsTimeSpec;
}

impl ToVfsTimeSpec for TimeSpec {
    fn into_vfs(self) -> VfsTimeSpec {
        VfsTimeSpec::new(self.tv_sec as u64, self.tv_nsec as u64)
    }
}

/// 获取当前计时器的值
#[inline]
pub fn read_timer() -> usize {
    arch::read_timer()
}

/// 获取当前时间，以 ms 为单位
pub fn get_time_ms() -> isize {
    (read_timer() / (CLOCK_FREQ / MSEC_PER_SEC)) as isize
}
