use alloc::sync::Arc;

use crate::driver::GenericBlockDevice;
use crate::error::AlienResult;
use crate::fs::dev::DeviceId;
use spin::Once;
use vfscore::error::VfsError;
use vfscore::file::VfsFile;
use vfscore::inode::{InodeAttr, VfsInode};
use vfscore::utils::{VfsFileStat, VfsNodeType, VfsPollEvents};
use vfscore::VfsResult;

use crate::interrupt::DeviceBase;

pub static BLOCK_DEVICE: Once<Arc<GenericBlockDevice>> = Once::new();

pub fn init_block_device(block_device: Arc<GenericBlockDevice>) {
    // BLOCK_DEVICE.lock().push(block_device);
    BLOCK_DEVICE.call_once(|| block_device);
}

pub trait BlockDevice: Send + Sync + DeviceBase {
    fn read(&self, buf: &mut [u8], offset: usize) -> AlienResult<usize>;
    fn write(&self, buf: &[u8], offset: usize) -> AlienResult<usize>;
    fn size(&self) -> usize;
    fn flush(&self) -> AlienResult<()>;
}

pub struct BLKDevice {
    device_id: DeviceId,
    device: Arc<GenericBlockDevice>,
}

impl BLKDevice {
    pub fn new(device_id: DeviceId, device: Arc<GenericBlockDevice>) -> Self {
        Self { device_id, device }
    }
    pub fn device_id(&self) -> DeviceId {
        self.device_id
    }
}

impl VfsFile for BLKDevice {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        self.device
            .read(buf, offset as usize)
            .map_err(|_| VfsError::IoError)
    }
    fn write_at(&self, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        self.device
            .write(buf, offset as usize)
            .map_err(|_| VfsError::IoError)
    }
    fn poll(&self, _event: VfsPollEvents) -> VfsResult<VfsPollEvents> {
        todo!()
    }
    fn ioctl(&self, _cmd: u32, _arg: usize) -> VfsResult<usize> {
        todo!()
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
            st_size: self.device.size() as u64,
            ..Default::default()
        })
    }
    fn inode_type(&self) -> VfsNodeType {
        VfsNodeType::BlockDevice
    }
}
