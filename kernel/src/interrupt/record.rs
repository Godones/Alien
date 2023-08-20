use crate::fs::vfs::VfsProvider;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::AtomicBool;
use lazy_static::lazy_static;
use rvfs::dentry::vfs_truncate_by_file;
use rvfs::file::{
    vfs_llseek, vfs_open_file, vfs_read_file, vfs_write_file, File, FileMode, FileMode2, OpenFlags,
    SeekFrom,
};
use spin::Mutex;

lazy_static! {
    pub static ref INTERRUPT_RECORD: Mutex<BTreeMap<u32, usize>> = {
        let mut tree = BTreeMap::new();
        tree.insert(1, 1);
        Mutex::new(tree)
    };
}

static FLAG: AtomicBool = AtomicBool::new(true);

pub fn set_flag(flag: bool) {
    FLAG.store(flag, core::sync::atomic::Ordering::Relaxed);
}

pub fn write_interrupt_record(file: Arc<File>, irq: usize) {
    file.access_inner().f_mode2 = FileMode2::from_bits_truncate(0x777);
    let mut new_buf = interrupts_info();
    vfs_truncate_by_file(file.clone(), 0).unwrap();
    vfs_write_file::<VfsProvider>(file.clone(), new_buf.as_bytes(), 0).unwrap();
    if FLAG.load(core::sync::atomic::Ordering::Relaxed) {
        file.access_inner().f_pos = 0;
    }
    file.access_inner().f_mode2 = FileMode2::from_bits_truncate(0x600);
}

pub fn interrupts_info() -> String {
    let mut interrupts = INTERRUPT_RECORD.lock();
    let irq = interrupts.keys().next().unwrap().clone();
    let value = interrupts.get_mut(&irq).unwrap().clone();
    interrupts.insert(irq, value + 1);
    let res = format!("{}: {}\n", irq, value);
    res
}
