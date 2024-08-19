use alloc::{string::String, sync::Arc, vec::Vec};

use unifs::inode::basic_file_stat;
use vfscore::{
    error::VfsError,
    file::VfsFile,
    impl_common_inode_default,
    inode::{InodeAttr, VfsInode},
    superblock::VfsSuperBlock,
    utils::{
        VfsFileStat, VfsInodeMode, VfsNodePerm, VfsNodeType, VfsRenameFlag, VfsTime, VfsTimeSpec,
    },
    VfsResult,
};

use super::*;
use crate::RamFsProvider;
pub struct RamFsSymLinkInode<T: Send + Sync, R: VfsRawMutex> {
    basic: UniFsInodeSame<T, R>,
    inner: lock_api::Mutex<R, String>,
    ext_attr: lock_api::Mutex<R, BTreeMap<String, String>>,
}

impl<T: RamFsProvider + 'static, R: VfsRawMutex + 'static> RamFsSymLinkInode<T, R> {
    pub fn new(
        sb: &Arc<UniFsSuperBlock<R>>,
        provider: T,
        inode_number: u64,
        sy_name: String,
    ) -> Self {
        Self {
            basic: UniFsInodeSame::new(
                sb,
                provider,
                inode_number,
                VfsNodePerm::from_bits_truncate(0o777),
            ),
            inner: lock_api::Mutex::new(sy_name),
            ext_attr: lock_api::Mutex::new(BTreeMap::new()),
        }
    }
    pub fn update_metadata<F, Res>(&self, f: F) -> Res
    where
        F: FnOnce(&UniFsInodeSame<T, R>) -> Res,
    {
        f(&self.basic)
    }
}

impl<T: RamFsProvider + 'static, R: VfsRawMutex + 'static> VfsFile for RamFsSymLinkInode<T, R> {}

impl<T: RamFsProvider + 'static, R: VfsRawMutex + 'static> VfsInode for RamFsSymLinkInode<T, R> {
    fn get_super_block(&self) -> VfsResult<Arc<dyn VfsSuperBlock>> {
        let res = self.basic.sb.upgrade().unwrap();
        Ok(res)
    }

    fn node_perm(&self) -> VfsNodePerm {
        self.basic.inner.lock().perm
    }

    fn readlink(&self, buf: &mut [u8]) -> VfsResult<usize> {
        let inner = self.inner.lock();
        let len = inner.as_bytes().len();
        let min_len = buf.len().min(len);
        buf[..min_len].copy_from_slice(&inner.as_bytes()[..min_len]);
        Ok(min_len)
    }

    fn set_attr(&self, attr: InodeAttr) -> VfsResult<()> {
        set_attr(&self.basic, attr);
        Ok(())
    }

    fn get_attr(&self) -> VfsResult<VfsFileStat> {
        let mut basic = basic_file_stat(&self.basic);
        basic.st_size = self.inner.lock().as_bytes().len() as u64;
        basic.st_mode = VfsInodeMode::from(
            VfsNodePerm::from_bits_truncate(basic.st_mode as u16),
            VfsNodeType::SymLink,
        )
        .bits();
        Ok(basic)
    }
    fn list_xattr(&self) -> VfsResult<Vec<String>> {
        let res = self.ext_attr.lock().keys().cloned().collect();
        Ok(res)
    }
    fn inode_type(&self) -> VfsNodeType {
        VfsNodeType::SymLink
    }

    impl_common_inode_default!();

    fn update_time(&self, time: VfsTime, now: VfsTimeSpec) -> VfsResult<()> {
        match time {
            VfsTime::ModifiedTime(t) => self.basic.inner.lock().mtime = t,
            VfsTime::AccessTime(t) => self.basic.inner.lock().atime = t,
        }
        self.basic.inner.lock().ctime = now;
        Ok(())
    }
}
