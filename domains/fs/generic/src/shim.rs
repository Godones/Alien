use alloc::sync::Arc;

use interface::{InodeID, VfsDomain};
use rref::{RRef, RRefVec};
use vfscore::{
    file::VfsFile,
    inode::{InodeAttr, VfsInode},
    utils::{VfsFileStat, VfsNodeType, VfsPollEvents},
    VfsResult,
};

pub struct MountDevShimInode {
    inode_id: InodeID,
    vfs_domain: Arc<dyn VfsDomain>,
}

impl MountDevShimInode {
    pub fn new(inode_id: InodeID, vfs_domain: Arc<dyn VfsDomain>) -> Self {
        Self {
            inode_id,
            vfs_domain,
        }
    }
}

impl VfsFile for MountDevShimInode {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let share_buf = RRefVec::new(0, buf.len());
        let (res, r) = self
            .vfs_domain
            .vfs_read_at(self.inode_id, offset, share_buf)
            .unwrap();
        buf.copy_from_slice(res.as_slice());
        Ok(r)
    }
    fn write_at(&self, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        let share_buf = RRefVec::from_slice(buf);
        let (_buf, w) = self
            .vfs_domain
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

impl VfsInode for MountDevShimInode {
    fn set_attr(&self, _attr: InodeAttr) -> VfsResult<()> {
        Ok(())
    }
    fn get_attr(&self) -> VfsResult<VfsFileStat> {
        let stat = RRef::new(VfsFileStat::default());
        let stat = self.vfs_domain.vfs_getattr(self.inode_id, stat).unwrap();
        Ok(*stat)
    }
    fn inode_type(&self) -> VfsNodeType {
        VfsNodeType::BlockDevice
    }
}
