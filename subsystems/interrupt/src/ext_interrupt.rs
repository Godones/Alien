use plic::Mode;

use crate::{DEVICE_TABLE, PLIC};
use arch::hart_id;

pub fn external_interrupt_handler() {
    let plic = PLIC.get().unwrap();
    let hart_id = hart_id();
    let irq = plic.claim(hart_id as u32, Mode::Supervisor);
    let table = DEVICE_TABLE.lock();
    let device = table
        .get(&(irq as usize))
        .or_else(|| panic!("no device for irq {}", irq))
        .unwrap();
    device.hand_irq();
    plic.complete(hart_id as u32, Mode::Supervisor, irq);
}
