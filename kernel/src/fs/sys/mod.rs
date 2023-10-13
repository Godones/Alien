use crate::fs::ProcOrSysFsProviderImpl;
use crate::ksync::Mutex;
use alloc::sync::Arc;
use dynfs::DynFsDirInode;
use vfscore::dentry::VfsDentry;
use vfscore::fstype::{MountFlags, VfsFsType};

mod cpu;
mod info;
pub type SysFsDirInodeImpl = DynFsDirInode<ProcOrSysFsProviderImpl, Mutex<()>>;

pub fn init_sysfs(sysfs: Arc<dyn VfsFsType>) -> Arc<dyn VfsDentry> {
    let root_dt = sysfs.i_mount(MountFlags::empty(), None, &[]).unwrap();
    // let root_inode = root_dt.inode().unwrap();
    // let root_inode = root_inode
    //     .downcast_arc::<SysFsDirInodeImpl>()
    //     .map_err(|_| VfsError::Invalid).unwrap();
    println!("sysfs init success");
    root_dt
}
