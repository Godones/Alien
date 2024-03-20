mod filesystem;
mod interrupt;
mod mem;
mod mounts;

use crate::tree::{CommonFsProviderImpl, FS};
use alloc::sync::Arc;
use core::ops::Index;
use dynfs::DynFsDirInode;
use filesystem::SystemSupportFS;
use interrupt::InterruptRecord;
use ksync::Mutex;
use mem::MemInfo;
use mounts::MountInfo;
use vfscore::dentry::VfsDentry;
use vfscore::error::VfsError;
use vfscore::fstype::VfsFsType;
use vfscore::path::VfsPath;

pub type ProcFsDirInodeImpl = DynFsDirInode<CommonFsProviderImpl, Mutex<()>>;

///
/// ```bash
/// |
/// |-- meminfo
/// |-- interrupts
/// |-- mounts
/// |-- filesystems
/// ```
// todo!(use ramfs instead of dynfs)
pub fn init_procfs(procfs: Arc<dyn VfsFsType>) -> Arc<dyn VfsDentry> {
    let root_dt = procfs.i_mount(0, "/proc", None, &[]).unwrap();
    let root_inode = root_dt.inode().unwrap();
    let root_inode = root_inode
        .downcast_arc::<ProcFsDirInodeImpl>()
        .map_err(|_| VfsError::Invalid)
        .unwrap();
    root_inode
        .add_file_manually("meminfo", Arc::new(MemInfo), "r--r--r--".into())
        .unwrap();
    root_inode
        .add_file_manually("interrupts", Arc::new(InterruptRecord), "r--r--r--".into())
        .unwrap();
    root_inode
        .add_file_manually("mounts", Arc::new(MountInfo), "r--r--r--".into())
        .unwrap();
    let support_fs = SystemSupportFS::new();
    root_inode
        .add_file_manually("filesystems", Arc::new(support_fs), "r--r--r--".into())
        .unwrap();

    root_inode
        .add_dir_manually("self", "r-xr-xr-x".into())
        .unwrap();

    let path = VfsPath::new(root_dt.clone(), root_dt.clone());
    let ramfs = FS.lock().index("ramfs").clone();
    let fake_ramfs = ramfs.i_mount(0, "/proc/self", None, &[]).unwrap();
    path.join("self").unwrap().mount(fake_ramfs, 0).unwrap();

    path.join("self/exe")
        .unwrap()
        .symlink("/bin/busybox")
        .unwrap();

    basic::println!("procfs init success");

    root_dt
}
