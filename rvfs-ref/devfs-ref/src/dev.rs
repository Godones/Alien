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
    RRefVec, VfsResult,
};

use crate::{DevInodeSameNew, DevKernelProvider, UniFsSuperBlock};

pub struct DevFsDevInode<T: Send + Sync, R: VfsRawMutex> {
    rdev: u64,
    basic: UniFsInodeSame<T, R>,
    ty: VfsNodeType,
}

impl<T: DevKernelProvider + 'static, R: VfsRawMutex + 'static> DevFsDevInode<T, R> {
    pub fn new(
        sb: &Arc<UniFsSuperBlock<R>>,
        provider: T,
        inode_number: u64,
        rdev: u64,
        ty: VfsNodeType,
    ) -> Self {
        Self {
            rdev,
            basic: UniFsInodeSame::new(
                sb,
                provider,
                inode_number,
                VfsNodePerm::from_bits_truncate(0o666),
            ),
            ty,
        }
    }

    pub fn real_dev(&self) -> VfsResult<Arc<dyn VfsInode>> {
        let dev = self.basic.provider.rdev2device(self.rdev);
        if dev.is_none() {
            return Err(VfsError::NoDev);
        }
        Ok(dev.unwrap())
    }
}

impl<T: DevKernelProvider + 'static, R: VfsRawMutex + 'static> VfsFile for DevFsDevInode<T, R> {
    fn read_at(&self, offset: u64, buf: RRefVec<u8>) -> VfsResult<(RRefVec<u8>, usize)> {
        self.real_dev()?.read_at(offset, buf)
    }
    fn write_at(&self, offset: u64, buf: &RRefVec<u8>) -> VfsResult<usize> {
        self.real_dev()?.write_at(offset, buf)
    }
    fn poll(&self, event: VfsPollEvents) -> VfsResult<VfsPollEvents> {
        self.real_dev()?.poll(event)
    }

    fn ioctl(&self, _cmd: u32, _arg: usize) -> VfsResult<usize> {
        self.real_dev()?.ioctl(_cmd, _arg)
    }
    fn flush(&self) -> VfsResult<()> {
        self.real_dev()?.flush()
    }

    fn fsync(&self) -> VfsResult<()> {
        self.real_dev()?.fsync()
    }
}

impl<T: DevKernelProvider + 'static, R: VfsRawMutex + 'static> VfsInode for DevFsDevInode<T, R> {
    fn get_super_block(&self) -> VfsResult<Arc<dyn VfsSuperBlock>> {
        let res = self.basic.sb.upgrade().ok_or(VfsError::Invalid);
        res.map(|sb| sb as Arc<dyn VfsSuperBlock>)
    }

    fn node_perm(&self) -> VfsNodePerm {
        self.basic.inner.lock().perm
    }

    fn set_attr(&self, _attr: InodeAttr) -> VfsResult<()> {
        todo!()
    }

    fn get_attr(&self) -> VfsResult<VfsFileStat> {
        // todo!(use real dev)
        let mut attr = basic_file_stat(&self.basic);
        attr.st_size = self.real_dev()?.get_attr()?.st_size;
        attr.st_blksize = self.real_dev()?.get_attr()?.st_blksize;
        attr.st_mode = VfsInodeMode::from(
            VfsNodePerm::from_bits_truncate(attr.st_mode as u16),
            self.ty,
        )
        .bits();
        Ok(attr)
    }

    fn list_xattr(&self) -> VfsResult<Vec<String>> {
        Err(VfsError::NoSys)
    }

    impl_file_inode_default!();

    fn inode_type(&self) -> VfsNodeType {
        self.ty
    }

    fn truncate(&self, _len: u64) -> VfsResult<()> {
        Err(VfsError::NoSys)
    }

    fn update_time(&self, time: VfsTime, now: VfsTimeSpec) -> VfsResult<()> {
        match time {
            VfsTime::ModifiedTime(t) => self.basic.inner.lock().mtime = t,
            VfsTime::AccessTime(t) => self.basic.inner.lock().atime = t,
        }
        self.basic.inner.lock().ctime = now;
        Ok(())
    }
}
