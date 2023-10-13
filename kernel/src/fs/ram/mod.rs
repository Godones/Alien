use crate::config::{RTC_TIME, UTC};
use alloc::sync::Arc;
use vfscore::dentry::VfsDentry;
use vfscore::fstype::{MountFlags, VfsFsType};
use vfscore::utils::VfsNodeType;

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
/// ```
pub fn init_ramfs(ramfs: Arc<dyn VfsFsType>) -> Arc<dyn VfsDentry> {
    let root_dt = ramfs.i_mount(MountFlags::empty(), None, &[]).unwrap();
    let root_inode = root_dt.inode().unwrap();
    root_inode
        .create("root", VfsNodeType::Dir, "rwx------".into(), None)
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
        .create("bin", VfsNodeType::Dir, "rwxr-xr-x".into(), None)
        .unwrap();
    println!("ramfs init success");

    root_dt
}
