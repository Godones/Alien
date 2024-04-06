use alloc::sync::Arc;

use vfscore::{dentry::VfsDentry, utils::VfsNodeType};

///
/// ```bash
/// |
/// |-- root
///   |-- .bashrc
/// |--var
///   |-- log
///   |-- tmp(ramfs)
///   |-- run
/// |-- etc
///   |-- passwd
///   |--localtime
///   |--adjtime
/// |-- dev  (devfs)
/// |-- proc (procfs)
/// |-- sys  (sysfs)
/// |-- bin  (fat32)
/// |-- tmp   (ramfs)
/// ```
pub fn init_ramfs(root_dt: &Arc<dyn VfsDentry>) {
    let root_inode = root_dt.inode().unwrap();
    let root = root_inode
        .create("root", VfsNodeType::Dir, "rwxr-xr-x".into(), None)
        .unwrap();
    let var = root_inode
        .create("var", VfsNodeType::Dir, "rwxr-xr-x".into(), None)
        .unwrap();
    var.create("log", VfsNodeType::Dir, "rwxrwxr-x".into(), None)
        .unwrap();
    var.create("tmp", VfsNodeType::Dir, "rwxrwxrwx".into(), None)
        .unwrap();
    var.create("run", VfsNodeType::Dir, "rwxrwxrwx".into(), None)
        .unwrap();
    let etc = root_inode
        .create("etc", VfsNodeType::Dir, "rwxr-xr-x".into(), None)
        .unwrap();
    let passwd = etc
        .create("passwd", VfsNodeType::File, "rw-r--r--".into(), None)
        .unwrap();
    let localtime = etc
        .create("localtime", VfsNodeType::File, "rw-r--r--".into(), None)
        .unwrap();
    let adjtime = etc
        .create("adjtime", VfsNodeType::File, "rw-r--r--".into(), None)
        .unwrap();

    passwd
        .write_at(0, b"root:x:0:0:root:/root:/bin/bash\n")
        .unwrap();
    localtime.write_at(0, UTC).unwrap();
    adjtime.write_at(0, RTC_TIME.as_bytes()).unwrap();

    root_inode
        .create("dev", VfsNodeType::Dir, "rwxr-xr-x".into(), None)
        .unwrap();
    root_inode
        .create("proc", VfsNodeType::Dir, "rwxr-xr-x".into(), None)
        .unwrap();
    root_inode
        .create("sys", VfsNodeType::Dir, "rwxr-xr-x".into(), None)
        .unwrap();
    root_inode
        .create("tmp", VfsNodeType::Dir, "rwxrwxrwx".into(), None)
        .unwrap();
    root_inode
        .create("tests", VfsNodeType::Dir, "rwxr-xr-x".into(), None)
        .unwrap();

    let _bashrc = root
        .create(".bashrc", VfsNodeType::File, "rwxrwxrwx".into(), None)
        .unwrap();

    basic::println!("ramfs init success");
}

/// localtime文件中保存的内容
pub const UTC: &[u8] = &[
    b'T', b'Z', b'i', b'f', b'2', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x1, 0, 0,
    0, 0x1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x1, 0, 0, 0, 0x4, 0, 0, 0, 0, 0, 0, b'U', b'T', b'C',
    0, 0, 0, b'T', b'Z', b'i', b'f', b'2', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0x1, 0, 0, 0, 0x1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x1, 0, 0, 0, 0x4, 0, 0, 0, 0, 0, 0, b'U',
    b'T', b'C', 0, 0, 0, 0x0a, 0x55, 0x54, 0x43, 0x30, 0x0a,
];

/// rtc文件中保存的内容
pub const RTC_TIME: &str = r"
rtc_time	: 03:01:50
rtc_date	: 2023-07-11
alrm_time	: 13:03:24
alrm_date	: 2023-07-11
alarm_IRQ	: no
alrm_pending	: no
update IRQ enabled	: no
periodic IRQ enabled	: no
periodic IRQ frequency	: 1024
max user IRQ frequency	: 64
24hr		: yes
periodic_IRQ	: no
update_IRQ	: no
HPET_emulated	: no
BCD		: yes
DST_enable	: no
periodic_freq	: 1024
batt_status	: okay";
