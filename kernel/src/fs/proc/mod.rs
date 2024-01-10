mod filesystem;
mod interrupt;
mod mem;
mod mounts;
mod process;

use crate::fs::proc::filesystem::SystemSupportFS;
use crate::fs::proc::interrupt::InterruptRecord;
use crate::fs::proc::mounts::MountInfo;
use crate::fs::CommonFsProviderImpl;
use crate::ksync::Mutex;
use alloc::sync::Arc;
use dynfs::DynFsDirInode;
use mem::MemInfo;
use vfscore::dentry::VfsDentry;
use vfscore::error::VfsError;
use vfscore::fstype::VfsFsType;

pub type ProcFsDirInodeImpl = DynFsDirInode<CommonFsProviderImpl, Mutex<()>>;

///
/// ```bash
/// |
/// |-- meminfo
/// |-- interrupts
/// |-- mounts
/// |-- filesystems
/// ```
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
    println!("procfs init success");

    root_dt
}
