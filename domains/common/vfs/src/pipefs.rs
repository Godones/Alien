use alloc::sync::Arc;

use dynfs::DynFsDirInode;
use ksync::Mutex;
use spin::Once;
use vfscore::dentry::VfsDentry;

use crate::tree::CommonFsProviderImpl;

#[allow(unused)]
type PipeFsDirInodeImpl = DynFsDirInode<CommonFsProviderImpl, Mutex<()>>;
pub static PIPE_FS_ROOT: Once<Arc<dyn VfsDentry>> = Once::new();

pub fn init_pipefs(root_dt: Arc<dyn VfsDentry>) {
    PIPE_FS_ROOT.call_once(|| root_dt);
    basic::println!("pipefs init success");
}
