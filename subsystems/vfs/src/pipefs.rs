use alloc::sync::Arc;

use constants::io::MountFlags;
use dynfs::DynFsDirInode;
use spin::Once;
use vfscore::{dentry::VfsDentry, fstype::VfsFsType};

use crate::CommonFsProviderImpl;

pub type PipeFsDirInodeImpl = DynFsDirInode<CommonFsProviderImpl, spin::Mutex<()>>;
pub static PIPE_FS_ROOT: Once<Arc<dyn VfsDentry>> = Once::new();

pub fn init_pipefs(fs: Arc<dyn VfsFsType>) {
    let root = fs
        .i_mount(MountFlags::empty().bits(), "", None, &[])
        .unwrap();
    PIPE_FS_ROOT.call_once(|| root);
    println!("pipefs init success");
}
