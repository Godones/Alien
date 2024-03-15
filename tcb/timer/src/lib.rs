#![no_std]

use constants::sys::TimeVal;
use platform::config::CLOCK_FREQ;
const TICKS_PER_SEC: usize = 10;
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

/// 更精细的时间，秒(s)+纳秒(ns)
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TimeSpec {
    pub tv_sec: usize,
    pub tv_nsec: usize, //0~999999999
}

impl TimeSpec {
    /// 创建一个新的 [`TimeSpec`] 时钟
    pub fn new(sec: usize, ns: usize) -> Self {
        Self {
            tv_sec: sec,
            tv_nsec: ns,
        }
    }

    /// 获取一个可以表示当前 cpu 时间的一个 [`TimeSpec`] 时钟
    pub fn now() -> Self {
        let time = arch::read_timer();
        Self {
            tv_sec: time / CLOCK_FREQ,
            tv_nsec: (time % CLOCK_FREQ) * 1000000000 / CLOCK_FREQ,
        }
    }

    /// 将本时钟所表示的时间间隔转化为 cpu 上时钟的跳变数
    pub fn to_clock(&self) -> usize {
        self.tv_sec * CLOCK_FREQ + self.tv_nsec * CLOCK_FREQ / 1000_000_000
    }
}

/// [`getitimer`] / [`setitimer`] 指定的类型，用户执行系统调用时获取和输入的计时器
#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct ITimerVal {
    /// 计时器超时间隔
    pub it_interval: TimeVal,
    /// 计时器当前所剩时间
    pub it_value: TimeVal,
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

/// 设置下一次时钟的中断
#[inline]
pub fn set_next_trigger() {
    let next = read_timer() + CLOCK_FREQ / TICKS_PER_SEC;
    assert!(next > read_timer());
    platform::set_timer(next);
}

/// 设置内核态中下一次时钟的中断
///
/// 原设计为内核态下的时间片设置的更短一些，以免一个进程在进入内核态前后占用过多的时间片。但目前修改为 内核态和用户态下的时间片大小相同。
#[inline]
pub fn set_next_trigger_in_kernel() {
    let next = read_timer() + CLOCK_FREQ / TICKS_PER_SEC;
    assert!(next > read_timer());
    platform::set_timer(next);
}
