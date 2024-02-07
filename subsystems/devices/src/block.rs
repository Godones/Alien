use alloc::sync::Arc;
use constants::DeviceId;
use device_interface::BlockDevice;
use spin::Once;
use vfscore::error::VfsError;
use vfscore::file::VfsFile;
use vfscore::inode::{InodeAttr, VfsInode};
use vfscore::utils::{VfsFileStat, VfsNodeType, VfsPollEvents};
use vfscore::VfsResult;

use drivers::block_device::GenericBlockDevice;
pub static BLOCK_DEVICE: Once<Arc<GenericBlockDevice>> = Once::new();

pub fn init_block_device(block_device: Arc<GenericBlockDevice>) {
    // BLOCK_DEVICE.lock().push(block_device);
    BLOCK_DEVICE.call_once(|| block_device);
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
            st_size: self.device.size() as u64,
            st_blksize: 512,
            ..Default::default()
        })
    }
    fn inode_type(&self) -> VfsNodeType {
        VfsNodeType::BlockDevice
    }
}
