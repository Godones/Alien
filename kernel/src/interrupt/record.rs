use crate::ksync::Mutex;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use spin::Lazy;

/// Record the number of interrupts
pub static INTERRUPT_RECORD: Lazy<Mutex<BTreeMap<usize, usize>>> = Lazy::new(|| {
    let mut tree = BTreeMap::new();
    tree.insert(1, 0); // timer
    tree.insert(10, 0); // uart
    Mutex::new(tree)
});

/// Increase the number of interrupts
pub fn write_irq_info(irq: usize) {
    let mut interrupts = INTERRUPT_RECORD.lock();
    let value = interrupts.get_mut(&irq).unwrap().clone();
    interrupts.insert(irq, value + 1);
}

/// Serializes the number of interrupts
///
/// # Return
/// irq{}: number
pub fn interrupts_info() -> String {
    let interrupts = INTERRUPT_RECORD.lock();
    let mut res = String::new();
    interrupts.iter().for_each(|(irq, value)| {
        res.push_str(&format!("{}: {}\r\n", irq, value));
    });
    res
}
