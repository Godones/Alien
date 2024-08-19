#![cfg_attr(not(test), no_std)]
#![feature(trait_alias)]
extern crate alloc;

mod inode;

use alloc::{
    string::{String, ToString},
    sync::{Arc, Weak},
    vec::Vec,
};

pub use inode::*;
use log::info;
use unifs::{dentry::UniFsDentry, *};
use vfscore::{
    dentry::VfsDentry,
    error::VfsError,
    fstype::{FileSystemFlags, VfsFsType},
    inode::VfsInode,
    superblock::VfsSuperBlock,
    utils::{VfsNodePerm, VfsTimeSpec},
    VfsResult,
};

pub trait RamFsProvider: Send + Sync + Clone {
    fn current_time(&self) -> VfsTimeSpec;
}

pub struct RamFs<T: Send + Sync, R: VfsRawMutex> {
    provider: T,
    fs_container: lock_api::Mutex<R, Vec<Arc<UniFs<T, R>>>>,
}

impl<T: RamFsProvider, R: VfsRawMutex + 'static> RamFs<T, R> {
    pub fn new(provider: T) -> Self {
        Self {
            provider,
            fs_container: lock_api::Mutex::new(Vec::new()),
        }
    }
}

impl<T: RamFsProvider + 'static, R: VfsRawMutex + 'static> VfsFsType for RamFs<T, R> {
    fn mount(
        self: Arc<Self>,
        _flags: u32,
        _ab_mnt: &str,
        _dev: Option<Arc<dyn VfsInode>>,
        _data: &[u8],
    ) -> VfsResult<Arc<dyn VfsDentry>> {
        let unifs = Arc::new(UniFs::<T, R>::new("ramfs", self.provider.clone()));
        let sb = UniFsSuperBlock::new(&(self.clone() as Arc<dyn VfsFsType>));
        let root = Arc::new(RamFsDirInode::new(
            &sb,
            self.provider.clone(),
            0,
            VfsNodePerm::from_bits_truncate(0o755),
        ));
        let parent = Weak::<UniFsDentry<R>>::new();
        sb.inode_index
            .fetch_add(1, core::sync::atomic::Ordering::SeqCst);
        sb.inode_count
            .fetch_add(1, core::sync::atomic::Ordering::SeqCst);
        sb.root.lock().replace(root.clone());
        unifs.sb.lock().replace(sb);
        self.fs_container.lock().push(unifs);
        Ok(Arc::new(UniFsDentry::<R>::root(root, parent)))
    }

    fn kill_sb(&self, sb: Arc<dyn VfsSuperBlock>) -> VfsResult<()> {
        let sb = sb
            .downcast_arc::<UniFsSuperBlock<R>>()
            .map_err(|_| VfsError::Invalid)?;

        let mut fs_container = self.fs_container.lock();

        if let Some((index, _)) = fs_container.iter().enumerate().find(|(_, fs)| {
            let isb = fs.sb.lock();
            isb.is_some() && Arc::ptr_eq(isb.as_ref().unwrap(), &sb)
        }) {
            info!("kill ramfs sb success");
            fs_container.remove(index);
            Ok(())
        } else {
            Err(VfsError::Invalid)
        }
    }
    fn fs_flag(&self) -> FileSystemFlags {
        FileSystemFlags::empty()
    }

    fn fs_name(&self) -> String {
        "ramfs".to_string()
    }
}
