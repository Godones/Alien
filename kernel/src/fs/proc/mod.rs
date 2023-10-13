mod interrupt;
mod mem;
mod process;

use crate::fs::proc::interrupt::InterruptRecord;
use crate::fs::ProcOrSysFsProviderImpl;
use crate::ksync::Mutex;
use alloc::sync::Arc;
use dynfs::DynFsDirInode;
use mem::MemInfo;
use vfscore::dentry::VfsDentry;
use vfscore::error::VfsError;
use vfscore::fstype::{MountFlags, VfsFsType};

pub type ProcFsDirInodeImpl = DynFsDirInode<ProcOrSysFsProviderImpl, Mutex<()>>;

///
/// ```bash
/// |
/// |-- meminfo
/// |-- interrupts
/// ```
pub fn init_procfs(procfs: Arc<dyn VfsFsType>) -> Arc<dyn VfsDentry> {
    let root_dt = procfs.i_mount(MountFlags::empty(), None, &[]).unwrap();
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

    println!("procfs init success");

    root_dt
}
