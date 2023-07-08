use alloc::collections::BinaryHeap;
use alloc::sync::Arc;
use core::cmp::Ordering;

use lazy_static::lazy_static;

use kernel_sync::Mutex;
use syscall_define::time::ClockId;
use syscall_table::syscall_func;

use crate::arch;
use crate::config::CLOCK_FREQ;
use crate::task::schedule::schedule;
use crate::task::{current_task, StatisticalData, Task, TaskState, TASK_MANAGER};

const TICKS_PER_SEC: usize = 100;
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

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TimeVal {
    /// seconds
    pub tv_sec: usize,
    /// microseconds
    pub tv_usec: usize,
}

impl TimeVal {
    pub fn now() -> Self {
        let time = read_timer();
        Self {
            tv_sec: time / CLOCK_FREQ,
            tv_usec: (time % CLOCK_FREQ) * 1000000 / CLOCK_FREQ,
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
    let task = current_task().unwrap().access_inner();
    let statistic_data = task.statistical_data();
    let time = Times::from_process_data(statistic_data);
    let tms = task.transfer_raw_ptr_mut(tms as *mut Times);
    // copy_to_user_buf(tv,&time);
    *tms = time;
    0
}

#[syscall_func(101)]
pub fn sys_nanosleep(req: *mut u8, _: *mut u8) -> isize {
    let task = current_task().unwrap();
    let req = task.transfer_raw_ptr(req as *mut TimeSpec);
    let end_time = read_timer() + req.tv_sec * CLOCK_FREQ + req.tv_nsec * CLOCK_FREQ / 1000000000;
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
    let tp = task.transfer_raw_ptr(tp as *mut TimeSpec);
    match id {
        ClockId::Realtime => {
            let time = TimeSpec::now();
            *tp = time;
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
