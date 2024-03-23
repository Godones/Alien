use alloc::sync::Arc;
use constants::DeviceId;
use interface::InputDomain;
use vfscore::error::VfsError;
use vfscore::file::VfsFile;
use vfscore::inode::{InodeAttr, VfsInode};
use vfscore::superblock::VfsSuperBlock;
use vfscore::utils::{VfsFileStat, VfsNodeType, VfsPollEvents};
use vfscore::VfsResult;

pub struct INPUTDevice {
    device_id: DeviceId,
    device: Arc<dyn InputDomain>,
    #[allow(unused)]
    is_keyboard: bool,
}

impl INPUTDevice {
    pub fn new(device_id: DeviceId, device: Arc<dyn InputDomain>, is_keyboard: bool) -> Self {
        Self {
            device_id,
            device,
            is_keyboard,
        }
    }
    pub fn device_id(&self) -> DeviceId {
        self.device_id
    }
}

impl VfsFile for INPUTDevice {
    fn read_at(&self, _offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        if buf.len() != 8 {
            return Err(VfsError::Invalid);
        }
        let event = self.device.event_block().unwrap();
        let event_bytes = event.to_be_bytes();
        buf.copy_from_slice(&event_bytes);
        Ok(1)
    }
    fn write_at(&self, _offset: u64, _buf: &[u8]) -> VfsResult<usize> {
        Err(VfsError::Invalid)
    }
    fn poll(&self, event: VfsPollEvents) -> VfsResult<VfsPollEvents> {
        let mut res = VfsPollEvents::empty();
        if event.contains(VfsPollEvents::IN) {
            if self.device.have_event().unwrap() {
                res |= VfsPollEvents::IN;
            }
        }
        Ok(res)
    }
}

impl VfsInode for INPUTDevice {
    fn get_super_block(&self) -> VfsResult<Arc<dyn VfsSuperBlock>> {
        Err(VfsError::NoSys)
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
