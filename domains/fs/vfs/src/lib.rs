#![no_std]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use core::ops::Index;
use interface::VfsDomain;
use ksync::Mutex;
use rref::RpcResult;
use spin::{Lazy, Once};
use vfscore::dentry::VfsDentry;
use vfscore::fstype::VfsFsType;
use vfscore::path::VfsPath;

pub static FS: Lazy<Mutex<BTreeMap<String, Arc<dyn VfsDomain>>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));

static SYSTEM_ROOT_FS: Once<Arc<dyn VfsDomain>> = Once::new();

fn register_all_fs() {
    let procfs = libsyscall::get_fs_domain("procfs").unwrap();
    let sysfs = libsyscall::get_fs_domain("sysfs").unwrap();
    let ramfs = libsyscall::get_fs_domain("ramfs").unwrap();
    let devfs = libsyscall::get_fs_domain("devfs").unwrap();
    let tmpfs = libsyscall::get_fs_domain("tmpfs").unwrap();
    let pipefs = libsyscall::get_fs_domain("pipefs").unwrap();
    let diskfs = libsyscall::get_fs_domain("fatfs").unwrap();

    FS.lock().insert("procfs".to_string(), procfs);
    FS.lock().insert("sysfs".to_string(), sysfs);
    FS.lock().insert("ramfs".to_string(), ramfs);
    FS.lock().insert("devfs".to_string(), devfs);
    FS.lock().insert("tmpfs".to_string(), tmpfs);
    FS.lock().insert("pipefs".to_string(), pipefs);
    FS.lock().insert("diskfs".to_string(), diskfs);

    libsyscall::println!("register fs success");
}

/// Init the filesystem
pub fn init_filesystem() -> RpcResult<()> {
    register_all_fs();
    let ramfs_root = ram::init_ramfs(FS.lock().index("ramfs").clone());
    let procfs = FS.lock().index("procfs").clone();
    let procfs_root = proc::init_procfs(procfs);
    let devfs_root = dev::init_devfs(FS.lock().index("devfs").clone());
    let sysfs_root = sys::init_sysfs(FS.lock().index("sysfs").clone());
    let tmpfs_root = FS
        .lock()
        .index("tmpfs")
        .clone()
        .i_mount(0, "/tmp", None, &[])?;

    pipefs::init_pipefs(FS.lock().index("pipefs").clone());

    let path = VfsPath::new(ramfs_root.clone(), ramfs_root.clone());
    path.join("proc")?.mount(procfs_root, 0)?;
    path.join("sys")?.mount(sysfs_root, 0)?;
    path.join("dev")?.mount(devfs_root, 0)?;
    path.join("tmp")?.mount(tmpfs_root.clone(), 0)?;

    let shm_ramfs = FS
        .lock()
        .index("ramfs")
        .clone()
        .i_mount(0, "/dev/shm", None, &[])?;
    path.join("dev/shm")?.mount(shm_ramfs, 0)?;

    let diskfs = FS.lock().index("diskfs").clone();
    let blk_inode = path
        .join("/dev/sda")?
        .open(None)
        .expect("open /dev/sda failed")
        .inode()?;
    let diskfs_root = diskfs.i_mount(0, "/tests", Some(blk_inode), &[])?;
    path.join("tests")?.mount(diskfs_root, 0)?;
    vfscore::path::print_fs_tree(&mut VfsOutPut, ramfs_root.clone(), "".to_string(), false)
        .unwrap();

    initrd::populate_initrd(ramfs_root.clone())?;

    SYSTEM_ROOT_FS.call_once(|| ramfs_root);
    println!("Init filesystem success");
    Ok(())
}

struct VfsOutPut;
impl core::fmt::Write for VfsOutPut {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        libsyscall::write_console(s);
        Ok(())
    }
}

/// Get the root filesystem of the system
#[inline]
pub fn system_root_fs() -> Arc<dyn VfsDentry> {
    SYSTEM_ROOT_FS.get().unwrap().clone()
}

/// Get the filesystem by name
#[inline]
pub fn system_support_fs(fs_name: &str) -> Option<Arc<dyn VfsFsType>> {
    FS.lock().iter().find_map(|(name, fs)| {
        if name == fs_name {
            Some(fs.clone())
        } else {
            None
        }
    })
}

pub fn main() -> Arc<dyn VfsDomain> {}
