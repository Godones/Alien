use interface::InodeID;
use rref::{RRef, RRefVec};
use vfscore::{
    file::VfsFile,
    inode::{InodeAttr, VfsInode},
    utils::{VfsFileStat, VfsNodeType, VfsPollEvents},
    VfsResult,
};

use crate::VFS_DOMAIN;

pub struct MountPointShimInode {
    inode_id: InodeID,
}

impl MountPointShimInode {
    pub fn new(inode_id: InodeID) -> Self {
        Self { inode_id }
    }
}

impl VfsFile for MountPointShimInode {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let share_buf = RRefVec::new(0, buf.len());
        let vfs_domain = VFS_DOMAIN.get().unwrap();
        let (res, r) = vfs_domain
            .vfs_read_at(self.inode_id, offset, share_buf)
            .unwrap();
        buf.copy_from_slice(res.as_slice());
        Ok(r)
    }
    fn write_at(&self, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        let vfs_domain = VFS_DOMAIN.get().unwrap();
        let share_buf = RRefVec::from_slice(buf);
        let (_buf, w) = vfs_domain
            .vfs_write_at(self.inode_id, offset, share_buf)
            .unwrap();
        Ok(w)
    }
    fn poll(&self, _event: VfsPollEvents) -> VfsResult<VfsPollEvents> {
        unimplemented!()
    }
    fn ioctl(&self, _cmd: u32, _arg: usize) -> VfsResult<usize> {
        unimplemented!()
    }
    fn flush(&self) -> VfsResult<()> {
        Ok(())
    }
    fn fsync(&self) -> VfsResult<()> {
        Ok(())
    }
}

impl VfsInode for MountPointShimInode {
    fn set_attr(&self, _attr: InodeAttr) -> VfsResult<()> {
        Ok(())
    }
    fn get_attr(&self) -> VfsResult<VfsFileStat> {
        let vfs_domain = VFS_DOMAIN.get().unwrap();
        let stat = RRef::new(VfsFileStat::default());
        let stat = vfs_domain.vfs_getattr(self.inode_id, stat).unwrap();
        Ok(*stat)
    }
    fn inode_type(&self) -> VfsNodeType {
        VfsNodeType::BlockDevice
    }
}
