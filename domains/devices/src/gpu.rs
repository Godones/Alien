use crate::DeviceId;
use alloc::sync::Arc;
use interface::GpuDomain;
use spin::Once;
use vfscore::error::VfsError;
use vfscore::file::VfsFile;
use vfscore::inode::{InodeAttr, VfsInode};
use vfscore::utils::{VfsFileStat, VfsNodeType};
use vfscore::VfsResult;

pub static GPU_DEVICE: Once<Arc<dyn GpuDomain>> = Once::new();

#[allow(unused)]
pub fn init_gpu(gpu: Arc<dyn GpuDomain>) {
    GPU_DEVICE.call_once(|| gpu);
}

pub struct GPUDevice {
    device_id: DeviceId,
    device: Arc<dyn GpuDomain>,
}

impl GPUDevice {
    pub fn new(device_id: DeviceId, device: Arc<dyn GpuDomain>) -> Self {
        Self { device_id, device }
    }
    pub fn device_id(&self) -> DeviceId {
        self.device_id
    }
}

impl VfsFile for GPUDevice {
    fn read_at(&self, _offset: u64, _buf: &mut [u8]) -> VfsResult<usize> {
        Err(VfsError::Invalid)
    }
    fn write_at(&self, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        let gbuf = self.device.get_framebuffer();
        let offset = offset as usize;
        let gbuf_len = gbuf.len();
        let min_len = (gbuf_len - offset).min(buf.len());
        gbuf[offset..offset + min_len].copy_from_slice(&buf[..min_len]);
        Ok(min_len)
    }
    fn flush(&self) -> VfsResult<()> {
        self.device.flush();
        Ok(())
    }
    fn fsync(&self) -> VfsResult<()> {
        self.flush()
    }
}

impl VfsInode for GPUDevice {
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
