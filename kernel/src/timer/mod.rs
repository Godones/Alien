use crate::arch;
use crate::config::CLOCK_FREQ;
use crate::task::schedule::schedule;
use crate::task::{current_process, Process, ProcessState, PROCESS_MANAGER};
use alloc::collections::BinaryHeap;
use alloc::sync::Arc;
use core::cmp::Ordering;
use lazy_static::lazy_static;
use spin::Mutex;
use syscall_table::syscall_func;

const TICKS_PER_SEC: usize = 100;
const MSEC_PER_SEC: usize = 1000;
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

#[syscall_func(169)]
pub fn get_time_ms() -> isize {
    (read_timer() / (CLOCK_FREQ / MSEC_PER_SEC)) as isize
}

#[syscall_func(1005)]
pub fn sleep(ms: usize) -> isize {
    let end_time = read_timer() + ms * (CLOCK_FREQ / MSEC_PER_SEC);
    if read_timer() < end_time {
        let process = current_process().unwrap();
        process.update_state(ProcessState::Sleeping);
        push_to_timer_queue(process.clone(), end_time);
        schedule();
    }
    0
}

#[derive(Debug)]
pub struct Timer {
    end_time: usize,
    process: Arc<Process>,
}
impl Timer {
    pub fn new(end_time: usize, process: Arc<Process>) -> Self {
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

pub fn push_to_timer_queue(process: Arc<Process>, end_time: usize) {
    TIMER_QUEUE.lock().push(Timer::new(end_time, process));
}

pub fn check_timer_queue() {
    let now = read_timer();
    let mut queue = TIMER_QUEUE.lock();
    while let Some(timer) = queue.peek() {
        if timer.end_time <= now {
            let timer = queue.pop().unwrap();
            PROCESS_MANAGER.lock().push_front(timer.process);
        } else {
            break;
        }
    }
}
