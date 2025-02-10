use alloc::{string::String, sync::Arc, vec::Vec};

use unifs::{
    inode::{basic_file_stat, UniFsInodeSame},
    *,
};
use vfscore::{
    error::VfsError,
    file::VfsFile,
    impl_file_inode_default,
    inode::{InodeAttr, VfsInode},
    superblock::VfsSuperBlock,
    utils::{
        VfsFileStat, VfsInodeMode, VfsNodePerm, VfsNodeType, VfsPollEvents, VfsRenameFlag, VfsTime,
        VfsTimeSpec,
    },
    DVec, VfsResult,
};

use crate::{DynFsKernelProvider, UniFsSuperBlock, UniInodeSameNew};

pub struct DynFsFileInode<T: Send + Sync, R: VfsRawMutex> {
    basic: UniFsInodeSame<T, R>,
    real_inode: Arc<dyn VfsInode>,
}

impl<T: DynFsKernelProvider + 'static, R: VfsRawMutex + 'static> DynFsFileInode<T, R> {
    pub fn new(
        sb: &Arc<UniFsSuperBlock<R>>,
        provider: T,
        inode_number: u64,
        real_inode: Arc<dyn VfsInode>,
        perm: VfsNodePerm,
    ) -> Self {
        Self {
            real_inode,
            basic: UniFsInodeSame::new(sb, provider, inode_number, perm),
        }
    }
    fn real_inode(&self) -> VfsResult<Arc<dyn VfsInode>> {
        Ok(self.real_inode.clone())
    }
}

impl<T: DynFsKernelProvider + 'static, R: VfsRawMutex + 'static> VfsFile for DynFsFileInode<T, R> {
    fn read_at(&self, offset: u64, buf: DVec<u8>) -> VfsResult<(DVec<u8>, usize)> {
        self.real_inode()?.read_at(offset, buf)
    }
    fn write_at(&self, offset: u64, buf: &DVec<u8>) -> VfsResult<usize> {
        self.real_inode()?.write_at(offset, buf)
    }
    fn poll(&self, event: VfsPollEvents) -> VfsResult<VfsPollEvents> {
        self.real_inode()?.poll(event)
    }

    fn ioctl(&self, _cmd: u32, _arg: usize) -> VfsResult<usize> {
        self.real_inode()?.ioctl(_cmd, _arg)
    }
    fn flush(&self) -> VfsResult<()> {
        self.real_inode()?.flush()
    }

    fn fsync(&self) -> VfsResult<()> {
        self.real_inode()?.fsync()
    }
}

impl<T: DynFsKernelProvider + 'static, R: VfsRawMutex + 'static> VfsInode for DynFsFileInode<T, R> {
    fn get_super_block(&self) -> VfsResult<Arc<dyn VfsSuperBlock>> {
        let res = self.basic.sb.upgrade().ok_or(VfsError::Invalid);
        res.map(|sb| sb as Arc<dyn VfsSuperBlock>)
    }

    fn node_perm(&self) -> VfsNodePerm {
        self.basic.inner.lock().perm
    }

    fn set_attr(&self, _attr: InodeAttr) -> VfsResult<()> {
        Ok(())
    }

    fn get_attr(&self) -> VfsResult<VfsFileStat> {
        let mut attr = basic_file_stat(&self.basic);
        let real_attr = self.real_inode()?.get_attr()?;
        attr.st_size = real_attr.st_size;
        attr.st_mode = VfsInodeMode::from(
            VfsNodePerm::from_bits_truncate(attr.st_mode as u16),
            VfsNodeType::File,
        )
        .bits();
        Ok(attr)
    }

    fn list_xattr(&self) -> VfsResult<Vec<String>> {
        Err(VfsError::NoSys)
    }

    fn inode_type(&self) -> VfsNodeType {
        VfsNodeType::File
    }

    impl_file_inode_default!();

    fn truncate(&self, _len: u64) -> VfsResult<()> {
        Err(VfsError::PermissionDenied)
    }
    fn update_time(&self, time: VfsTime, now: VfsTimeSpec) -> VfsResult<()> {
        self.real_inode()?.update_time(time, now)
    }
}
