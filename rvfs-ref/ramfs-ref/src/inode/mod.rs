mod dir;
mod file;
mod symlink;

use alloc::{collections::BTreeMap, string::String, sync::Arc};

pub use dir::RamFsDirInode;
pub use file::RamFsFileInode;
use unifs::{
    inode::{UniFsInodeAttr, UniFsInodeSame},
    UniFsSuperBlock,
};
use vfscore::{
    inode::InodeAttr,
    utils::{VfsFileStat, VfsNodePerm},
};

use super::VfsRawMutex;
use crate::RamFsProvider;

trait RamFsInodeSameNew<T: Send + Sync, R: VfsRawMutex> {
    fn new(sb: &Arc<UniFsSuperBlock<R>>, provider: T, inode_number: u64, perm: VfsNodePerm)
        -> Self;
}

impl<T: RamFsProvider + 'static, R: VfsRawMutex + 'static> RamFsInodeSameNew<T, R>
    for UniFsInodeSame<T, R>
{
    fn new(
        sb: &Arc<UniFsSuperBlock<R>>,
        provider: T,
        inode_number: u64,
        perm: VfsNodePerm,
    ) -> Self {
        let time = provider.current_time();
        Self {
            sb: Arc::downgrade(sb),
            inode_number,
            provider,
            inner: lock_api::Mutex::new(UniFsInodeAttr {
                link_count: 1,
                atime: time,
                mtime: time,
                ctime: time,
                perm,
            }),
        }
    }
}

fn set_attr<T: Send + Sync, R: VfsRawMutex>(basic: &UniFsInodeSame<T, R>, attr: InodeAttr) {
    let mut inner = basic.inner.lock();
    inner.atime = attr.atime;
    inner.mtime = attr.mtime;
    inner.ctime = attr.ctime;
}
