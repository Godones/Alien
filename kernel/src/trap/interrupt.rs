//! Alien 的外部中断处理
//!
//! 目前仅有时钟中断处理函数。
use crate::interrupt::record::write_interrupt_record;
use crate::ipc::solve_futex_wait;
use crate::task::do_suspend;
use crate::timer::{check_timer_queue, set_next_trigger};

/// 时钟中断处理函数
pub fn timer_interrupt_handler() {
    write_interrupt_record(0);
    check_timer_queue();
    solve_futex_wait();
    set_next_trigger();
    do_suspend();
}
