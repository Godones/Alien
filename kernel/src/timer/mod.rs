use alloc::collections::BinaryHeap;
use alloc::sync::Arc;
use core::cmp::Ordering;

use lazy_static::lazy_static;

use kernel_sync::Mutex;
use syscall_define::sys::TimeVal;
use syscall_define::time::{ClockId, TimerType};
use syscall_define::LinuxErrno;
use syscall_table::syscall_func;

use crate::arch;
use crate::config::CLOCK_FREQ;
use crate::task::schedule::schedule;
use crate::task::{current_task, do_suspend, StatisticalData, Task, TaskState, TASK_MANAGER};

const TICKS_PER_SEC: usize = 100;
const TICKS_PER_SEC_IN_KERNEL: usize = 1000;

const MSEC_PER_SEC: usize = 1000;

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

    pub fn from_process_data(data: &StatisticalData) -> Self {
        Self {
            tms_stime: data.tms_stime,
            tms_utime: data.tms_utime,
            tms_cstime: data.tms_cstime,
            tms_cutime: data.tms_cutime,
        }
    }
}

pub trait TimeNow {
    fn now() -> Self;
}

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

impl TimeFromFreq for TimeVal {
    fn from_freq(freq: usize) -> Self {
        Self {
            tv_sec: freq / CLOCK_FREQ,
            tv_usec: (freq % CLOCK_FREQ) * 1000000 / CLOCK_FREQ,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TimeSpec {
    pub tv_sec: usize,
    pub tv_nsec: usize, //0~999999999
}

impl TimeSpec {
    pub fn new(sec: usize, ns: usize) -> Self {
        Self {
            tv_sec: sec,
            tv_nsec: ns,
        }
    }
    pub fn now() -> Self {
        let time = read_timer();
        Self {
            tv_sec: time / CLOCK_FREQ,
            tv_nsec: (time % CLOCK_FREQ) * 1000000000 / CLOCK_FREQ,
        }
    }

    pub fn to_clock(&self) -> usize {
        self.tv_sec * CLOCK_FREQ + self.tv_nsec * CLOCK_FREQ / 1000000000
    }
}

impl TimeFromFreq for TimeSpec {
    fn from_freq(freq: usize) -> Self {
        Self {
            tv_sec: freq / CLOCK_FREQ,
            tv_nsec: (freq % CLOCK_FREQ) * 1000000000 / CLOCK_FREQ,
        }
    }
}

/// gettimer / settimer 指定的类型，用户输入输出计时器
#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct ITimerVal {
    pub it_interval: TimeVal,
    pub it_value: TimeVal,
}

/// 获取当前计时器的值
#[inline]
pub fn read_timer() -> usize {
    arch::read_timer()
}

/// 设置下一次时钟的中断
#[inline]
pub fn set_next_trigger() {
    crate::sbi::set_timer(read_timer() + CLOCK_FREQ / TICKS_PER_SEC);
}

#[inline]
pub fn set_next_trigger_in_kernel() {
    crate::sbi::set_timer(read_timer() + CLOCK_FREQ / TICKS_PER_SEC_IN_KERNEL);
}

// #[syscall_func(169)]
pub fn get_time_ms() -> isize {
    (read_timer() / (CLOCK_FREQ / MSEC_PER_SEC)) as isize
}

/// Reference: https://man7.org/linux/man-pages/man2/gettimeofday.2.html
#[syscall_func(169)]
pub fn get_time_of_day(tv: *mut u8) -> isize {
    let time = TimeVal::now();
    let process = current_task().unwrap();
    let tv = process.transfer_raw_ptr(tv as *mut TimeVal);
    *tv = time;
    0
}

/// Reference: https://man7.org/linux/man-pages/man2/times.2.html
#[syscall_func(153)]
pub fn times(tms: *mut u8) -> isize {
    let mut task = current_task().unwrap().access_inner();
    let statistic_data = task.statistical_data();
    let time = Times::from_process_data(statistic_data);
    task.copy_to_user(&time, tms as *mut Times);
    0
}

#[syscall_func(101)]
pub fn nanosleep(req: *mut u8, _: *mut u8) -> isize {
    let task = current_task().unwrap();
    let req = task.transfer_raw_ptr(req as *mut TimeSpec);
    let end_time = read_timer() + req.to_clock();
    if read_timer() < end_time {
        let process = current_task().unwrap();
        process.update_state(TaskState::Sleeping);
        push_to_timer_queue(process.clone(), end_time);
        schedule();
    }
    0
}

#[syscall_func(113)]
pub fn clock_get_time(clock_id: usize, tp: *mut u8) -> isize {
    let id = ClockId::from_raw(clock_id).unwrap();
    let task = current_task().unwrap();
    match id {
        ClockId::Monotonic | ClockId::Realtime => {
            let time = TimeSpec::now();
            task.access_inner().copy_to_user(&time, tp as *mut TimeSpec)
        }
        _ => {
            panic!("clock_get_time: clock_id {:?} not supported", id);
        }
    }
    0
}

#[derive(Debug)]
pub struct Timer {
    end_time: usize,
    process: Arc<Task>,
}

impl Timer {
    pub fn new(end_time: usize, process: Arc<Task>) -> Self {
        Self { end_time, process }
    }
    pub fn get_task(&self) -> &Arc<Task> {
        &self.process
    }
}

impl PartialEq for Timer {
    fn eq(&self, other: &Self) -> bool {
        self.end_time == other.end_time
    }
}

impl Eq for Timer {}

impl PartialOrd for Timer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // reverse order
        Some(other.end_time.cmp(&self.end_time))
    }
}

impl Ord for Timer {
    fn cmp(&self, other: &Self) -> Ordering {
        // reverse order
        other.end_time.cmp(&self.end_time)
    }
}

lazy_static! {
    pub static ref TIMER_QUEUE: Mutex<BinaryHeap<Timer>> = Mutex::new(BinaryHeap::new());
}

pub fn push_to_timer_queue(process: Arc<Task>, end_time: usize) {
    TIMER_QUEUE.lock().push(Timer::new(end_time, process));
}

pub fn check_timer_queue() {
    let now = read_timer();
    let mut queue = TIMER_QUEUE.lock();
    while let Some(timer) = queue.peek() {
        if timer.end_time <= now {
            let timer = queue.pop().unwrap();
            TASK_MANAGER.lock().push_front(timer.process);
        } else {
            break;
        }
    }
}

#[syscall_func(102)]
pub fn getitimer(_which: usize, current_value: usize) -> isize {
    let task = current_task().unwrap();
    let timer = &task.access_inner().timer;
    let itimer = ITimerVal {
        it_interval: timer.timer_interval_us.into(),
        it_value: timer.timer_remained_us.into(),
    };
    task.access_inner()
        .copy_to_user(&itimer, current_value as *mut ITimerVal);
    0
}

#[syscall_func(103)]
pub fn setitimer(which: usize, current_value: usize, old_value: usize) -> isize {
    let which = TimerType::try_from(which).unwrap();
    assert_ne!(which, TimerType::NONE);
    warn!(
        "setitimer: which {:?} ,curret_value {:#x}, old_value {:#x}",
        which, current_value, old_value
    );
    let task = current_task().unwrap();
    if old_value != 0 {
        let timer = task.access_inner().get_timer();
        let itimer = ITimerVal {
            it_interval: timer.timer_interval_us.into(),
            it_value: timer.timer_remained_us.into(),
        };
        task.access_inner()
            .copy_to_user(&itimer, old_value as *mut ITimerVal);
    }
    assert_ne!(current_value, 0);
    let mut itimer = ITimerVal::default();
    task.access_inner()
        .copy_from_user(current_value as *const ITimerVal, &mut itimer);
    task.access_inner().set_timer(itimer, which);
    0
}

#[syscall_func(114)]
pub fn clock_getres(id: usize, res: usize) -> isize {
    let id = ClockId::from_raw(id).unwrap();
    warn!("clock_getres: id {:?} ,res {:#x}", id, res);
    let task = current_task().unwrap();
    let time_res = match id {
        ClockId::Monotonic => {
            let time = TimeSpec::new(0, 1);
            time
        }
        _ => {
            panic!("clock_get_time: clock_id {:?} not supported", id);
        }
    };
    task.access_inner()
        .copy_to_user(&time_res, res as *mut TimeSpec);
    0
}

#[syscall_func(115)]
pub fn clock_nanosleep(clock_id: usize, flags: usize, req: usize, remain: usize) -> isize {
    const TIMER_ABSTIME: usize = 1;
    let id = ClockId::from_raw(clock_id).unwrap();
    warn!(
        "clock_nanosleep: id {:?} ,flags {:#x}, req {:#x}, remain {:#x}",
        id, flags, req, remain
    );
    let task = current_task().unwrap().clone();
    match id {
        ClockId::Monotonic => {
            assert_eq!(flags, TIMER_ABSTIME);
            let mut target_time = TimeSpec::new(0, 0);
            task.access_inner()
                .copy_from_user(req as *const TimeSpec, &mut target_time);
            let end_time = target_time.to_clock();

            loop {
                let now = read_timer();
                if now >= end_time {
                    break;
                }
                do_suspend();
                // check signal
                let task_inner = task.access_inner();
                let receiver = task_inner.signal_receivers.lock();
                if receiver.have_signal() {
                    return LinuxErrno::EINTR.into();
                }
            }
        }
        _ => {
            panic!("clock_nanotime: clock_id {:?} not supported", id);
        }
    }
    0
}
