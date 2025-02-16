#![cfg_attr(not(test), no_std)]
extern crate alloc;

mod dir;
mod file;

use alloc::{string::String, sync::Arc};

pub use dir::DynFsDirInode;
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

pub trait DynFsKernelProvider: Send + Sync + Clone {
    fn current_time(&self) -> VfsTimeSpec;
}

pub struct DynFs<T: Send + Sync, R: VfsRawMutex>(UniFs<T, R>);

impl<T: DynFsKernelProvider + 'static, R: VfsRawMutex + 'static> DynFs<T, R> {
    pub fn new(provider: T, fs_name: &'static str) -> Self {
        Self(UniFs::new(fs_name, provider))
    }
}

impl<T: DynFsKernelProvider + 'static, R: VfsRawMutex + 'static> VfsFsType for DynFs<T, R> {
    fn mount(
        self: Arc<Self>,
        _flags: u32,
        ab_mnt: &str,
        _dev: Option<Arc<dyn VfsInode>>,
        _data: &[u8],
    ) -> VfsResult<Arc<dyn VfsDentry>> {
        let fs = self.clone() as Arc<dyn VfsFsType>;
        let mut this = self.0.sb.lock();
        if this.is_none() {
            let sb = UniFsSuperBlock::new(&fs, self.0.magic());
            let root = Arc::new(DynFsDirInode::new(
                0,
                self.0.provider.clone(),
                &sb,
                VfsNodePerm::from_bits_truncate(0o755),
            ));
            *sb.root.lock() = Some(root.clone());
            sb.inode_index
                .fetch_add(1, core::sync::atomic::Ordering::SeqCst);
            sb.inode_count
                .fetch_add(1, core::sync::atomic::Ordering::SeqCst);
            this.replace(sb);
            this.as_ref().unwrap().root_dentry(ab_mnt)
        } else {
            this.as_ref().unwrap().root_dentry(ab_mnt)
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

trait UniInodeSameNew<T: Send + Sync, R: VfsRawMutex> {
    fn new(sb: &Arc<UniFsSuperBlock<R>>, provider: T, inode_number: u64, perm: VfsNodePerm)
        -> Self;
}

impl<T: DynFsKernelProvider + 'static, R: VfsRawMutex + 'static> UniInodeSameNew<T, R>
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
