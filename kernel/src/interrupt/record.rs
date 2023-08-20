use crate::fs::vfs::VfsProvider;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use lazy_static::lazy_static;
use rvfs::dentry::vfs_truncate_by_file;
use rvfs::file::{vfs_open_file, vfs_read_file, vfs_write_file, FileMode, FileMode2, OpenFlags, vfs_llseek};
use spin::Mutex;

lazy_static! {
    pub static ref INTERRUPT_RECORD: Mutex<BTreeMap<u32, usize>> = {
        let mut tree = BTreeMap::new();
        tree.insert(1, 0);
        Mutex::new(tree)
    };
}

pub fn write_interrupt_record(irq: usize) -> String {
    let file = vfs_open_file::<VfsProvider>(
        "/proc/interrupts",
        OpenFlags::O_RDWR | OpenFlags::O_CREAT,
        FileMode::FMODE_RDWR,
    )
        .unwrap();
    let mut buf = [0u8; 512];
    file.access_inner().f_mode2 = FileMode2::from_bits_truncate(0x777);
    let len = vfs_read_file::<VfsProvider>(file.clone(), &mut buf, 0).unwrap();
    let buf = String::from_utf8(buf[..len].to_vec()).unwrap();
    let lines = buf
        .split("\n")
        .map(|x| x.to_string())
        .collect::<Vec<String>>();
    // remove empty line
    let lines = lines
        .into_iter()
        .filter(|x| x.len() > 0)
        .collect::<Vec<String>>();
    // println!("lines: {:?}", lines);
    let mut new_buf = String::new();
    for line in lines {
        if line.starts_with(&format!("{}:", irq)) {
            let mut line = line
                .split_whitespace()
                .map(|x| x.to_string())
                .collect::<Vec<String>>();
            let mut count = line[1].parse::<usize>().unwrap();
            count += 1;
            line[1] = format!("{}", count);
            let line = line.join(" ");
            new_buf.push_str(&line);
            new_buf.push_str("\n");
        } else {
            new_buf.push_str(&line);
            new_buf.push_str("\n");
        }
    }
    vfs_truncate_by_file(file.clone(), 0).unwrap();
    vfs_write_file::<VfsProvider>(file.clone(), new_buf.as_bytes(), 0).unwrap();
    file.access_inner().f_mode2 = FileMode2::from_bits_truncate(0x600);
    new_buf
}

pub fn interrupts_info() -> String {
    let mut interrupts = INTERRUPT_RECORD.lock();
    let irq = interrupts.keys().next().unwrap().clone();
    let value = interrupts.get_mut(&irq).unwrap().clone();
    interrupts.remove(&irq);
    interrupts.insert(irq, value + 1);
    let res = format!("{}: {}", irq, value);
    res
}
