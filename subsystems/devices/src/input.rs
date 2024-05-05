use alloc::sync::Arc;

use constants::DeviceId;
use device_interface::InputDevice;
use spin::Once;
use vfscore::{
    error::VfsError,
    file::VfsFile,
    inode::{InodeAttr, VfsInode},
    superblock::VfsSuperBlock,
    utils::{VfsFileStat, VfsNodeType, VfsPollEvents},
    VfsResult,
};

pub struct INPUTDevice {
    device_id: DeviceId,
    device: Arc<dyn InputDevice>,
    #[allow(unused)]
    is_keyboard: bool,
}

impl INPUTDevice {
    pub fn new(device_id: DeviceId, device: Arc<dyn InputDevice>, is_keyboard: bool) -> Self {
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
        let buf = unsafe { core::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u64, 1) };
        let event = self.device.read_event_async();
        buf[0] = event;
        Ok(1)
    }
    fn write_at(&self, _offset: u64, _buf: &[u8]) -> VfsResult<usize> {
        Err(VfsError::Invalid)
    }
    fn poll(&self, event: VfsPollEvents) -> VfsResult<VfsPollEvents> {
        let mut res = VfsPollEvents::empty();
        if event.contains(VfsPollEvents::IN) {
            if !self.device.is_empty() {
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

pub static KEYBOARD_INPUT_DEVICE: Once<Arc<dyn InputDevice>> = Once::new();
pub static MOUSE_INPUT_DEVICE: Once<Arc<dyn InputDevice>> = Once::new();

pub fn init_keyboard_input_device(input_device: Arc<dyn InputDevice>) {
    KEYBOARD_INPUT_DEVICE.call_once(|| input_device);
}

pub fn init_mouse_input_device(input_device: Arc<dyn InputDevice>) {
    MOUSE_INPUT_DEVICE.call_once(|| input_device);
}
