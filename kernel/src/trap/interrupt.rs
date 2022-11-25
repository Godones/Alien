use crate::config::TIMER_FREQ;
use crate::timer::set_next_trigger;

/// 时钟中断处理函数
pub fn timer_interrupt_handler() {
    // per s
    set_next_trigger(TIMER_FREQ);
}
