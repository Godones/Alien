use crate::CommonFsProviderImpl;
use alloc::sync::Arc;
use dynfs::DynFsDirInode;
use ksync::Mutex;
use vfscore::dentry::VfsDentry;
use vfscore::fstype::VfsFsType;

#[allow(unused)]
type SysFsDirInodeImpl = DynFsDirInode<CommonFsProviderImpl, Mutex<()>>;

pub fn init_sysfs(sysfs: Arc<dyn VfsFsType>) -> Arc<dyn VfsDentry> {
    let root_dt = sysfs.i_mount(0, "/sys", None, &[]).unwrap();
    // let root_inode = root_dt.inode().unwrap();
    // let root_inode = root_inode
    //     .downcast_arc::<SysFsDirInodeImpl>()
    //     .map_err(|_| VfsError::Invalid).unwrap();
    libsyscall::println!("sysfs init success");
    root_dt
}
