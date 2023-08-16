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
use crate::task::{current_task, do_suspend, StatisticalData, Task, TASK_MANAGER};

const TICKS_PER_SEC: usize = 10;
// const TICKS_PER_SEC_IN_KERNEL: usize = 1000;

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

pub trait ToClock {
    fn to_clock(&self) -> usize;
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
        self.tv_sec * CLOCK_FREQ + self.tv_nsec * CLOCK_FREQ / 1000_000_000
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

/// 设置下一次时钟的中断
#[inline]
pub fn set_next_trigger() {
    let next = read_timer() + CLOCK_FREQ / TICKS_PER_SEC;
    assert!(next > read_timer());
    crate::sbi::set_timer(next);
}

#[inline]
pub fn set_next_trigger_in_kernel() {
    let next = read_timer() + CLOCK_FREQ / TICKS_PER_SEC;
    assert!(next > read_timer());
    crate::sbi::set_timer(next);
}

// #[syscall_func(169)]
pub fn get_time_ms() -> isize {
    (read_timer() / (CLOCK_FREQ / MSEC_PER_SEC)) as isize
}

/// 一个系统调用函数，获取当前的时间，获取的时间将存储在`tv`所指向的[`TimeVal`]结构处。
/// 执行成功则返回0。
/// 
/// Reference: [get_time_of_day](https://man7.org/linux/man-pages/man2/gettimeofday.2.html)
#[syscall_func(169)]
pub fn get_time_of_day(tv: *mut u8) -> isize {
    let time = TimeVal::now();
    let process = current_task().unwrap();
    let tv = process.transfer_raw_ptr(tv as *mut TimeVal);
    *tv = time;
    0
}

/// 一个系统调用函数，获取当前进程在用户态/内核态下运行的时间、最后一次运行在用户态/内核态下的时间等，
/// 获取的信息将保存在`tms`所指向的[`Times`]结构处。执行成功返回0。
///
/// Reference: [times](https://man7.org/linux/man-pages/man2/times.2.html)
#[syscall_func(153)]
pub fn times(tms: *mut u8) -> isize {
    let mut task = current_task().unwrap().access_inner();
    let statistic_data = task.statistical_data();
    let time = Times::from_process_data(statistic_data);
    task.copy_to_user(&time, tms as *mut Times);
    0
}

/// 一个系统调用函数，暂停本进程直到一段时间后结束，要暂停的时间将保存在`req`所指向的[`TimeSpec`]结构处。
/// 但在`nanosleep`执行过程中，本进程有可能被其他信号唤醒。
/// 函数若正常停止`req`时间则返回0；如果由于因为其他信号而被唤醒，此时函数返回-1(EINTR)。
///
/// Reference: [nanosleep](https://man7.org/linux/man-pages/man2/nanosleep.2.html)
#[syscall_func(101)]
pub fn nanosleep(req: *mut u8, _: *mut u8) -> isize {
    let task = current_task().unwrap().clone();
    let mut time = TimeSpec::new(0, 0);
    task.access_inner()
        .copy_from_user(req as *const TimeSpec, &mut time);
    warn!("nanosleep: {:?}", time);
    let end_time = read_timer() + time.to_clock();
    loop {
        if read_timer() >= end_time {
            break;
        }
        do_suspend();
        // interrupt by signal
        let task_inner = task.access_inner();
        let receiver = task_inner.signal_receivers.lock();
        if receiver.have_signal() {
            return LinuxErrno::EINTR as isize;
        }
    }

    0
}

/// 一个系统调用函数，可以根据输入的时钟类型`clock_id`来获取当前的时间，获取的时间将存储在`tp`所指向的[`TimeSpec`]结构处。
/// 
/// 目前仅支持`Monotonic`、`Realtime`和`ProcessCputimeId`三种时钟类型，均会返回当前的系统时间。
/// 执行成功则返回0；当所输入的`clock_id`不在`Monotonic`、`Realtime`和`ProcessCputimeId`中时，进程将会被panic。
///
/// Reference: [clock_get_time](https://www.man7.org/linux/man-pages/man3/clock_gettime.3.html)
#[syscall_func(113)]
pub fn clock_get_time(clock_id: usize, tp: *mut u8) -> isize {
    let id = ClockId::from_raw(clock_id).unwrap();
    let task = current_task().unwrap();
    match id {
        ClockId::Monotonic | ClockId::Realtime | ClockId::ProcessCputimeId => {
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

/// 一个系统调用函数，用于获取当前进程的计时器，保存在`current_value`指向的[`ITimerVal`]结构处。
/// 由于Alien目前每个进程只支持一个计时器，原定于分辨计时器种类的`_which`在此处并没有派上用场。
/// 函数执行成功则返回0。
/// Reference: [getitimer](https://man7.org/linux/man-pages/man2/setitimer.2.html)
#[syscall_func(102)]
pub fn getitimer(_which: usize, current_value: usize) -> isize {
    let task = current_task().unwrap();
    let timer = &task.access_inner().timer;
    let itimer = ITimerVal {
        it_interval: timer.timer_interval.into(),
        it_value: timer.timer_remained.into(),
    };
    task.access_inner()
        .copy_to_user(&itimer, current_value as *mut ITimerVal);
    0
}

/// 一个系统调用函数，用于将当前进程的定时器设置为`current_value`指向的[`ITimerVal`]结构处，
/// 同时将旧计时器的信息保存在`old_value`指向的[`ITimerVal`]结构处。
///
/// `which`参数需为目前支持的[`TimerType`]类型且不为`NONE`，否则会导致进程被panic。
/// 如果`current_value`为空，则会导致进程被panic。
/// 如果`old_value`为空，则不进行保存旧计时器信息操作。
///
/// 函数执行正确则返回0。
/// Reference: [setitimer](https://man7.org/linux/man-pages/man2/setitimer.2.html)
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
            it_interval: timer.timer_interval.into(),
            it_value: timer.timer_remained.into(),
        };
        task.access_inner()
            .copy_to_user(&itimer, old_value as *mut ITimerVal);
    }
    assert_ne!(current_value, 0);
    let mut itimer = ITimerVal::default();
    task.access_inner()
        .copy_from_user(current_value as *const ITimerVal, &mut itimer);
    error!("setitimer: itimer {:x?}", itimer);
    task.access_inner().set_timer(itimer, which);
    0
}

/// 一个系统调用函数，可以根据输入的时钟类型`clock_id`来获取该时钟分辨率(精度)，获取的精度将存储在`res`所指向的[`TimeSpec`]结构处。
/// 时钟的分辨率取决于实现方式，无法由特定进程配置。目前Alien仅支持`Monotonic`一种时钟类型。
///
/// Reference: [clock_getres](https://www.man7.org/linux/man-pages/man3/clock_getres.3.html)
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

/// 一个系统调用函数，如`nanosleep`一样，暂停本进程直到一段时间后结束，但`clock_nanosleep`可以根据传入的`clock_id`来指定使用的时钟类型。
/// 
/// 要暂停的时间将保存在`req`所指向的[`TimeSpec`]结构处。目前仅支持`Monotonic`，输入其它时钟类型将会返回使得进程panic。
/// 如`nanosleep`一样，在`clock_nanosleep`执行过程中，本进程也有可能被其他信号唤醒。
/// 
/// 函数若正常停止`req`时间则返回0；如果由于因为其他信号而被唤醒，此时函数返回-1(EINTR)。
///
/// Reference: [times](https://man7.org/linux/man-pages/man2/times.2.html)
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
