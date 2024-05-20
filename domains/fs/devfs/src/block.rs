use alloc::sync::Arc;

use basic::constants::DeviceId;
use interface::CacheBlkDeviceDomain;
use rref::RRefVec;
use vfscore::{
    error::VfsError,
    file::VfsFile,
    inode::{InodeAttr, VfsInode},
    utils::{VfsFileStat, VfsNodeType, VfsPollEvents},
    VfsResult,
};

pub struct BLKDevice {
    device_id: DeviceId,
    device: Arc<dyn CacheBlkDeviceDomain>,
}

impl BLKDevice {
    pub fn new(device_id: DeviceId, device: Arc<dyn CacheBlkDeviceDomain>) -> Self {
        Self { device_id, device }
    }
}

impl VfsFile for BLKDevice {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let share_buf = RRefVec::new(0, buf.len());
        let res = self
            .device
            .read(offset, share_buf)
            .map_err(|_| VfsError::IoError)?;
        buf.copy_from_slice(res.as_slice());
        Ok(buf.len())
    }
    fn write_at(&self, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        let share_buf = RRefVec::from_slice(buf);
        self.device
            .write(offset, &share_buf)
            .map_err(|_| VfsError::IoError)?;
        Ok(buf.len())
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

impl VfsInode for BLKDevice {
    fn set_attr(&self, _attr: InodeAttr) -> VfsResult<()> {
        Ok(())
    }
    fn get_attr(&self) -> VfsResult<VfsFileStat> {
        Ok(VfsFileStat {
            st_rdev: self.device_id.id(),
            st_size: self.device.get_capacity().unwrap() as u64,
            st_blksize: 512,
            ..Default::default()
        })
    }
    fn inode_type(&self) -> VfsNodeType {
        VfsNodeType::BlockDevice
    }
}
