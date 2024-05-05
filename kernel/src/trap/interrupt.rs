//! Alien 的外部中断处理
//!
//! 目前仅有时钟中断处理函数。
use interrupt::record_irq;

use crate::{
    ipc::solve_futex_wait,
    task::do_suspend,
    time::{check_timer_queue, set_next_trigger},
};

/// 时钟中断处理函数
pub fn timer_interrupt_handler() {
    record_irq(1);
    check_timer_queue();
    solve_futex_wait();
    set_next_trigger();
    do_suspend();
}
