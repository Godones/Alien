use alloc::sync::Arc;

use dynfs::DynFsDirInode;
use ksync::Mutex;
use vfscore::dentry::VfsDentry;

use crate::tree::CommonFsProviderImpl;

#[allow(unused)]
type SysFsDirInodeImpl = DynFsDirInode<CommonFsProviderImpl, Mutex<()>>;

pub fn init_sysfs(root_dt: Arc<dyn VfsDentry>) -> Arc<dyn VfsDentry> {
    basic::println!("sysfs init success");
    root_dt
}
