use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use spin::{Lazy, Mutex};

pub static INTERRUPT_RECORD: Lazy<Mutex<BTreeMap<usize, usize>>> = Lazy::new(|| {
    let mut tree = BTreeMap::new();
    tree.insert(1, 1); // timer
    tree.insert(10, 1); // uart
    Mutex::new(tree)
});

pub fn write_irq_info(irq: usize) {
    let mut interrupts = INTERRUPT_RECORD.lock();
    let value = interrupts.get_mut(&irq).unwrap().clone();
    interrupts.insert(irq, value + 1);
}

pub fn interrupts_info() -> String {
    let interrupts = INTERRUPT_RECORD.lock();
    let mut res = String::new();
    interrupts.iter().for_each(|(irq, value)| {
        res.push_str(&format!("{}: {}\r\n", irq, value));
    });
    res
}
