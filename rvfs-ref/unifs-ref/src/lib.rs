#![feature(trait_alias)]
#![cfg_attr(not(test), no_std)]

pub mod dentry;
pub mod inode;

extern crate alloc;

use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
    sync::{Arc, Weak},
};
use core::sync::atomic::{AtomicU64, AtomicUsize};

use log::info;
use vfscore::{
    dentry::VfsDentry,
    error::VfsError,
    fstype::{FileSystemFlags, VfsFsType},
    inode::VfsInode,
    superblock::{SuperType, VfsSuperBlock},
    utils::{VfsFsStat, VfsTimeSpec},
    VfsResult,
};

use crate::dentry::UniFsDentry;

pub trait VfsRawMutex = lock_api::RawMutex + Send + Sync;
pub struct UniFs<T: Send + Sync, R: VfsRawMutex> {
    real_fs: &'static str,
    pub provider: T,
    pub sb: lock_api::Mutex<R, Option<Arc<UniFsSuperBlock<R>>>>,
    magic: u128,
}

impl<T: Send + Sync, R: VfsRawMutex + 'static> UniFs<T, R> {
    pub fn new(name: &'static str, provider: T) -> Self {
        Self {
            real_fs: name,
            provider,
            sb: lock_api::Mutex::new(None),
            magic: uuid::Uuid::new_v4().as_u128(),
        }
    }

    pub fn magic(&self) -> u128 {
        self.magic
    }
}

impl<T: Send + Sync, R: VfsRawMutex + 'static> UniFs<T, R> {
    pub fn kill_sb(&self, sb: Arc<dyn VfsSuperBlock>) -> VfsResult<()> {
        let t_sb = sb
            .downcast_arc::<UniFsSuperBlock<R>>()
            .map_err(|_| VfsError::Invalid)?;
        let mut sb = self.sb.lock();
        if sb.is_none() {
            return Err(VfsError::Invalid);
        }
        let old_sb = sb.as_ref().unwrap();
        if !Arc::ptr_eq(old_sb, &t_sb) {
            return Err(VfsError::Invalid);
        }
        *sb = None;
        info!("{} killed", self.real_fs);
        Ok(())
    }
    pub fn fs_flag(&self) -> FileSystemFlags {
        FileSystemFlags::empty()
    }
    pub fn fs_name(&self) -> String {
        self.real_fs.to_string()
    }
}

pub struct UniFsSuperBlock<R: VfsRawMutex> {
    fs_type: Weak<dyn VfsFsType>,
    pub root: lock_api::Mutex<R, Option<Arc<dyn VfsInode>>>,
    pub inode_index: AtomicU64,
    pub inode_count: AtomicUsize,
    inode_cache: lock_api::Mutex<R, BTreeMap<u64, Arc<dyn VfsInode>>>,
    pub mnt_info: lock_api::Mutex<R, BTreeMap<String, Arc<dyn VfsDentry>>>,
    magic: u128,
}

impl<R: VfsRawMutex + 'static> UniFsSuperBlock<R> {
    /// Call this function only once
    pub fn new(fs_type: &Arc<dyn VfsFsType>, magic: u128) -> Arc<Self> {
        Arc::new(Self {
            fs_type: Arc::downgrade(fs_type),
            root: lock_api::Mutex::new(None),
            inode_index: AtomicU64::new(0),
            inode_count: AtomicUsize::new(0),
            inode_cache: lock_api::Mutex::new(BTreeMap::new()),
            mnt_info: lock_api::Mutex::new(BTreeMap::new()),
            magic,
        })
    }
    pub fn insert_inode(&self, inode_number: u64, inode: Arc<dyn VfsInode>) {
        let mut cache = self.inode_cache.lock();
        cache.insert(inode_number, inode);
    }
    pub fn remove_inode(&self, inode_number: u64) {
        let mut cache = self.inode_cache.lock();
        cache.remove(&inode_number);
    }
    pub fn get_inode(&self, inode_number: u64) -> Option<Arc<dyn VfsInode>> {
        let cache = self.inode_cache.lock();
        cache.get(&inode_number).cloned()
    }
    pub fn root_dentry(&self, ab_mnt: &str) -> VfsResult<Arc<dyn VfsDentry>> {
        let mut mnt_info = self.mnt_info.lock();
        let res = mnt_info.get(ab_mnt).cloned();
        match res {
            None => {
                let parent = Weak::<UniFsDentry<R>>::new();
                let inode = self.root.lock().clone().unwrap();
                let new = Arc::new(UniFsDentry::<R>::root(inode, parent));
                mnt_info.insert(ab_mnt.into(), new.clone());
                Ok(new as Arc<dyn VfsDentry>)
            }
            Some(x) => Ok(x),
        }
    }
}

impl<R: VfsRawMutex + 'static> VfsSuperBlock for UniFsSuperBlock<R> {
    fn sync_fs(&self, _wait: bool) -> VfsResult<()> {
        Ok(())
    }

    fn stat_fs(&self) -> VfsResult<VfsFsStat> {
        Ok(VfsFsStat {
            f_type: 0,
            f_bsize: 4096,
            f_blocks: (usize::MAX / 4096) as u64,
            f_bfree: (usize::MAX / 4096) as u64,
            f_bavail: (usize::MAX / 4096) as u64,
            f_files: (usize::MAX / 4096 / 4096) as u64,
            f_ffree: (usize::MAX / 4096 / 4096) as u64
                - self.inode_count.load(core::sync::atomic::Ordering::SeqCst) as u64,
            f_fsid: [0, 0],
            f_namelen: 255,
            f_frsize: 0,
            f_flags: 0,
            f_spare: [0; 4],
        })
    }

    fn super_type(&self) -> SuperType {
        SuperType::Single
    }

    fn fs_type(&self) -> Arc<dyn VfsFsType> {
        self.fs_type.upgrade().unwrap()
    }

    fn root_inode(&self) -> VfsResult<Arc<dyn VfsInode>> {
        let lock = self.root.lock();
        if let Some(root) = &*lock {
            Ok(root.clone())
        } else {
            Err(VfsError::Invalid)
        }
    }
    fn magic(&self) -> u128 {
        self.magic
    }
}
