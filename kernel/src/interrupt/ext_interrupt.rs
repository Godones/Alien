use plic::Mode;

use crate::arch::hart_id;
use crate::interrupt::{DEVICE_TABLE, PLIC};

pub fn external_interrupt_handler() {
    let plic = PLIC.get().unwrap();
    let hart_id = hart_id();
    let irq = plic.claim(hart_id as u32, Mode::Supervisor).unwrap().get();
    info!("external interrupt {} handled", irq);
    let table = DEVICE_TABLE.lock();
    let device = table
        .get(&(irq as usize))
        .or_else(|| panic!("no device for irq {}", irq))
        .unwrap();
    device.hand_irq();
    plic.complete(hart_id as u32, Mode::Supervisor, irq);
}
