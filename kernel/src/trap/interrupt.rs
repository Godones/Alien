use plic::Mode;

use crate::arch::hart_id;
use crate::driver::{DEVICE_TABLE, PLIC};
use crate::task::{current_task, TaskState};
use crate::task::schedule::schedule;
use crate::timer::{check_timer_queue, set_next_trigger};

/// 时钟中断处理函数
pub fn timer_interrupt_handler() {
    check_timer_queue();
    set_next_trigger();
    let process = current_task().unwrap();
    process.update_state(TaskState::Ready);
    schedule();
}

pub fn external_interrupt_handler() {
    let plic = PLIC.get().unwrap();
    let hart_id = hart_id();
    let irq = plic.claim(hart_id as u32, Mode::Supervisor).unwrap().get();
    let table = DEVICE_TABLE.lock();
    let device = table
        .get(&(irq as usize))
        .or_else(|| panic!("no device for irq {}", irq))
        .unwrap();
    device.hand_irq();
    plic.complete(hart_id as u32, Mode::Supervisor, irq);
}
