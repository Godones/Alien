#![cfg_attr(not(test), no_std)]
extern crate alloc;

mod dev;
mod dir;

use alloc::{string::String, sync::Arc};

use unifs::{
    inode::{UniFsInodeAttr, UniFsInodeSame},
    UniFs, UniFsSuperBlock, VfsRawMutex,
};
use vfscore::{
    dentry::VfsDentry,
    fstype::{FileSystemFlags, VfsFsType},
    inode::VfsInode,
    superblock::VfsSuperBlock,
    utils::{VfsNodePerm, VfsTimeSpec},
    VfsResult,
};

use crate::dir::DevFsDirInode;

pub trait DevKernelProvider: Send + Sync + Clone {
    fn current_time(&self) -> VfsTimeSpec;
    fn rdev2device(&self, rdev: u64) -> Option<Arc<dyn VfsInode>>;
}

pub struct DevFs<T: Send + Sync, R: VfsRawMutex>(UniFs<T, R>);

impl<T: DevKernelProvider + 'static, R: VfsRawMutex + 'static> DevFs<T, R> {
    pub fn new(provider: T) -> Self {
        Self(UniFs::new("devfs", provider))
    }
}

impl<T: DevKernelProvider + 'static, R: VfsRawMutex + 'static> VfsFsType for DevFs<T, R> {
    fn mount(
        self: Arc<Self>,
        _flags: u32,
        ab_mnt: &str,
        _dev: Option<Arc<dyn VfsInode>>,
        _data: &[u8],
    ) -> VfsResult<Arc<dyn VfsDentry>> {
        if self.0.sb.lock().is_none() {
            let sb = UniFsSuperBlock::new(&(self.clone() as Arc<dyn VfsFsType>));
            let root = Arc::new(DevFsDirInode::new(
                0,
                self.0.provider.clone(),
                &sb,
                VfsNodePerm::from_bits_truncate(0o755),
            ));
            *sb.root.lock() = Some(root);
            sb.inode_index
                .fetch_add(1, core::sync::atomic::Ordering::SeqCst);
            sb.inode_count
                .fetch_add(1, core::sync::atomic::Ordering::SeqCst);
            self.0.sb.lock().replace(sb.clone());
            sb.root_dentry(ab_mnt)
        } else {
            self.0.sb.lock().as_ref().unwrap().root_dentry(ab_mnt)
        }
    }

    fn kill_sb(&self, sb: Arc<dyn VfsSuperBlock>) -> VfsResult<()> {
        self.0.kill_sb(sb)
    }

    fn fs_flag(&self) -> FileSystemFlags {
        self.0.fs_flag()
    }

    fn fs_name(&self) -> String {
        self.0.fs_name()
    }
}

trait DevInodeSameNew<T: Send + Sync, R: VfsRawMutex> {
    fn new(sb: &Arc<UniFsSuperBlock<R>>, provider: T, inode_number: u64, perm: VfsNodePerm)
        -> Self;
}

impl<T: DevKernelProvider + 'static, R: VfsRawMutex + 'static> DevInodeSameNew<T, R>
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
