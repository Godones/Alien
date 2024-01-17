//! Alien 中的有关时钟、计时器的结构 以及 一些计时器的系统调用。
//!
//! 在对系统时间的记录上，Alien 中使用 [`TimeVal`] 记录 (秒，微秒) 的时间，使用 [`TimeSpec`] 记录 更精细的 (秒，纳秒) 的时间；
//! 在对进程的运行时间的记录上，使用 [`Times`] 结构记录进程运行的时间，记录的信息包括程序在用户态、内核态下分别运行的时间，
//! 其子进程运行的总时间等，在任务控制块中记录相应数据的结构为 [`StatisticalData`]。
//!
//! 计时器方面， [`Timer`] 结构为实际放入计时器队列 [`TIMER_QUEUE`] 中的计时器结构。
//! 当发生时钟中断时，会检查所有计时器队列中的计时器是否超时，具体可见 [`check_timer_queue`]。
//! [`ITimerVal`] 结构为系统调用 [`getitimer`] / [`setitimer`] 指定的类型，用户执行系统调用时获取和输入时需要为该种类型的计时器,
//! 在任务控制块中记录相应数据的字段为 `timer`(结构为 `TaskTimer` )。
//!
//! 对于时间片 (每次引发时钟中断的时间间隔) 大小的设计：目前 Alien 中用户态和内核态下采用相同的时间片间隔，1s 内触发 10 次时钟中断。
use crate::task::{current_task, do_suspend, StatisticalData};
use constants::sys::TimeVal;
use constants::time::{ClockId, TimerType};
use constants::LinuxErrno;
use log::{info, warn};
use platform::config::CLOCK_FREQ;
use platform::set_timer;
use syscall_table::syscall_func;
use timer::{read_timer, ITimerVal, TimeNow, TimeSpec, Times};

/// 每秒包含的 时间片 数，每隔一个时间片，就会产生一个时钟中断
const TICKS_PER_SEC: usize = 10;
// const TICKS_PER_SEC_IN_KERNEL: usize = 1000;

/// 设置下一次时钟的中断
#[inline]
pub fn set_next_trigger() {
    let next = read_timer() + CLOCK_FREQ / TICKS_PER_SEC;
    assert!(next > read_timer());
    set_timer(next);
}

/// 设置内核态中下一次时钟的中断
///
/// 原设计为内核态下的时间片设置的更短一些，以免一个进程在进入内核态前后占用过多的时间片。但目前修改为 内核态和用户态下的时间片大小相同。
#[inline]
pub fn set_next_trigger_in_kernel() {
    let next = read_timer() + CLOCK_FREQ / TICKS_PER_SEC;
    assert!(next > read_timer());
    set_timer(next);
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
    let time = times_from_process_data(statistic_data);
    task.copy_to_user(&time, tms as *mut Times);
    0
}

/// 从一个 [`StatisticalData`] 结构 (一般为 task 的 statistical_data 字段) 得到一个 `Times` 变量
pub fn times_from_process_data(data: &StatisticalData) -> Times {
    Times {
        tms_stime: data.tms_stime,
        tms_utime: data.tms_utime,
        tms_cstime: data.tms_cstime,
        tms_cutime: data.tms_cutime,
    }
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

/// 当发生时钟中断时，`trap_handler` 会调用该函数检查所有计时器队列中的计时器，并唤醒等待在这些计时器上的进程
///
/// 遍历所有计时器队列 [`TIMER_QUEUE`] 中的计时器，若计时器的超时时间在当前时间之前(即已超时)，那么将该等待的进程加入
/// 线程池的首位，马上对其进行调度。
pub fn check_timer_queue() {}

/// 一个系统调用函数，用于获取当前进程的计时器，保存在`current_value`指向的[`ITimerVal`]结构处。
/// 由于Alien目前每个进程只支持一个计时器，原定于分辨计时器种类的`_which`在此处并没有派上用场。
/// 函数执行成功则返回0。
/// Reference: [getitimer](https://man7.org/linux/man-pages/man2/setitimer.2.html)
#[syscall_func(102)]
pub fn getitimer(_which: usize, current_value: usize) -> isize {
    let task = current_task().unwrap();
    let timer = &task.access_inner().timer;
    let itimer = ITimerVal {
        it_interval: timer.timer_interval,
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
    info!(
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
    info!("setitimer: itimer {:x?}", itimer);
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
    info!("clock_getres: id {:?} ,res {:#x}", id, res);
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
    info!(
        "clock_nanosleep: id {:?} ,flags {:#x}, req {:#x}, remain {:#x}",
        id, flags, req, remain
    );
    match id {
        ClockId::Monotonic => {
            assert_eq!(flags, TIMER_ABSTIME);
            let mut target_time = TimeSpec::new(0, 0);
            let task = current_task().unwrap().clone();
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
