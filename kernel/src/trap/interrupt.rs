use crate::ipc::solve_futex_wait;
use crate::task::do_suspend;
use crate::timer::{check_timer_queue, set_next_trigger};

/// 时钟中断处理函数
pub fn timer_interrupt_handler() {
    check_timer_queue();
    solve_futex_wait();
    set_next_trigger();
    do_suspend();
}
