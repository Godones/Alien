use alloc::sync::Arc;

use basic::constants::DeviceId;
use interface::EmptyDeviceDomain;
use rref::RRefVec;
use vfscore::{
    error::VfsError,
    file::VfsFile,
    inode::{InodeAttr, VfsInode},
    superblock::VfsSuperBlock,
    utils::{VfsFileStat, VfsNodePerm, VfsNodeType},
    VfsResult,
};

// like null/random device
pub struct EmptyDevice {
    device_id: DeviceId,
    domain: Arc<dyn EmptyDeviceDomain>,
}
impl EmptyDevice {
    pub fn new(device_id: DeviceId, domain: Arc<dyn EmptyDeviceDomain>) -> Self {
        Self { device_id, domain }
    }
}

impl VfsFile for EmptyDevice {
    fn read_at(&self, _offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let shared_buf = RRefVec::new(0, buf.len());
        let shared_buf = self.domain.read(shared_buf)?;
        buf.copy_from_slice(shared_buf.as_slice());
        Ok(buf.len())
    }
    fn write_at(&self, _offset: u64, buf: &[u8]) -> VfsResult<usize> {
        Ok(buf.len())
    }
}

impl VfsInode for EmptyDevice {
    fn get_super_block(&self) -> VfsResult<Arc<dyn VfsSuperBlock>> {
        Err(VfsError::NoSys)
    }

    fn node_perm(&self) -> VfsNodePerm {
        VfsNodePerm::empty()
    }

    fn set_attr(&self, _attr: InodeAttr) -> VfsResult<()> {
        Ok(())
    }

    fn get_attr(&self) -> VfsResult<VfsFileStat> {
        Ok(VfsFileStat {
            st_rdev: self.device_id.id(),
            ..Default::default()
        })
    }

    fn inode_type(&self) -> VfsNodeType {
        VfsNodeType::CharDevice
    }
}
