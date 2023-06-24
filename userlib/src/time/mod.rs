use crate::syscall::{sys_get_time, sys_nanosleep};

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

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct TimeVal {
    /// seconds
    pub tv_sec: usize,
    /// microseconds
    pub tv_usec: usize,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct TimeSpec {
    pub tv_sec: usize,
    pub tv_nsec: usize, //0~999999999
}

impl From<TimeVal> for TimeSpec {
    fn from(tv: TimeVal) -> Self {
        Self {
            tv_sec: tv.tv_sec,
            tv_nsec: tv.tv_usec * 1000,
        }
    }
}


impl TimeVal {
    pub fn now() -> Self {
        let mut tv = TimeVal::default();
        get_time_of_day(&mut tv);
        tv
    }
}

pub fn get_time_ms() -> isize {
    let mut tv = TimeVal::default();
    let res = sys_get_time(&mut tv as *mut TimeVal as *mut u8);
    if res != 0 {
        return 0;
    }
    tv.tv_sec as isize * 1000 + tv.tv_usec as isize / 1000
}

pub fn get_time_of_day(tv: &mut TimeVal) -> isize {
    let res = sys_get_time(tv as *mut TimeVal as *mut u8);
    if res != 0 {
        return 0;
    }
    1
}

pub fn sleep(ms: usize) {
    let mut ts = TimeSpec::default();
    ts.tv_sec = ms / 1000;
    ts.tv_nsec = (ms % 1000) * 1000000;
    sys_nanosleep(&mut ts as *mut TimeSpec as *mut u8, 0 as *mut u8);
}
