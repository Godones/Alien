use crate::fs::vfs::VfsProvider;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::sync::Arc;
use rvfs::file::{vfs_write_file, File, FileMode2};
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

pub fn write_interrupt_record(file: Arc<File>) {
    file.access_inner().f_mode2 = FileMode2::from_bits_truncate(0x777);
    let new_buf = interrupts_info();
    let old_pos = file.access_inner().f_pos;
    vfs_write_file::<VfsProvider>(file.clone(), new_buf.as_bytes(), 0).unwrap();
    file.access_inner().f_mode2 = FileMode2::from_bits_truncate(0x600);
    file.access_inner().f_pos = old_pos;
}

fn interrupts_info() -> String {
    let interrupts = INTERRUPT_RECORD.lock();
    let mut res = String::new();
    interrupts.iter().for_each(|(irq, value)| {
        res.push_str(&format!("{}: {}\n", irq, value));
    });
    res
}
