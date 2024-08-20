#![cfg_attr(not(test), no_std)]
extern crate alloc;

use alloc::{string::String, sync::Arc};

use unifs::{UniFs, UniFsSuperBlock, VfsRawMutex};
use vfscore::{
    dentry::VfsDentry,
    fstype::{FileSystemFlags, VfsFsType},
    inode::VfsInode,
    superblock::VfsSuperBlock,
    utils::VfsTimeSpec,
    VfsResult,
};

pub trait FsKernelProvider: Send + Sync + Clone {
    fn current_time(&self) -> VfsTimeSpec;
}

pub struct CustomFs<T: Send + Sync, R: VfsRawMutex> {
    fs: UniFs<T, R>,
    root_inode: Arc<dyn VfsInode>,
}

impl<T: FsKernelProvider + 'static, R: VfsRawMutex + 'static> CustomFs<T, R> {
    pub fn new(provider: T, fs_name: &'static str, root_inode: Arc<dyn VfsInode>) -> Self {
        Self {
            fs: UniFs::new(fs_name, provider),
            root_inode,
        }
    }
}

impl<T: FsKernelProvider + 'static, R: VfsRawMutex + 'static> VfsFsType for CustomFs<T, R> {
    fn mount(
        self: Arc<Self>,
        _flags: u32,
        ab_mnt: &str,
        _dev: Option<Arc<dyn VfsInode>>,
        _data: &[u8],
    ) -> VfsResult<Arc<dyn VfsDentry>> {
        let fs = self.clone() as Arc<dyn VfsFsType>;
        let mut this = self.fs.sb.lock();
        if this.is_none() {
            let sb = UniFsSuperBlock::new(&fs);
            let root = self.root_inode.clone();
            *sb.root.lock() = Some(root);
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
        self.fs.kill_sb(sb)
    }

    fn fs_flag(&self) -> FileSystemFlags {
        self.fs.fs_flag()
    }

    fn fs_name(&self) -> String {
        self.fs.fs_name()
    }
}
