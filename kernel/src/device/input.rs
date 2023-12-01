use alloc::sync::Arc;

use spin::Once;

use crate::fs::dev::DeviceId;
use syscall_table::syscall_func;
use vfscore::error::VfsError;
use vfscore::file::VfsFile;
use vfscore::inode::{InodeAttr, VfsInode};
use vfscore::superblock::VfsSuperBlock;
use vfscore::utils::{VfsFileStat, VfsNodeType, VfsPollEvents};
use vfscore::VfsResult;

use crate::interrupt::DeviceBase;
use crate::task::current_task;

pub trait InputDevice: Send + Sync + DeviceBase {
    fn is_empty(&self) -> bool;
    fn read_event_with_block(&self) -> u64;
    fn read_event_without_block(&self) -> Option<u64>;
}

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
        if buf.len() != 8{
            return Err(VfsError::Invalid);
        }
        let buf = unsafe { core::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u64, 1) };
        let event = self.device.read_event_with_block();
        buf[0] = event;
        Ok(1)
    }
    fn write_at(&self, _offset: u64, _buf: &[u8]) -> VfsResult<usize> {
        Err(VfsError::Invalid)
    }
    fn poll(&self, event: VfsPollEvents) -> VfsResult<VfsPollEvents> {
        let mut res = VfsPollEvents::empty();
        if event.contains(VfsPollEvents::IN) {
            if self.device.is_empty() {
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

/// 一个系统调用函数，用于获取鼠标和键盘事件。
///
/// `sys_event_get`会将获取到的事件将保存在event_buf所指向的内存位置处，
/// 此次允许获取到的事件的最大值(即event_buf)的大小由len指出。
///
/// 函数将返回成功获取到的事件个数。
///
#[syscall_func(2002)]
pub fn sys_event_get(event_buf: *mut u64, len: usize) -> isize {
    let process = current_task().unwrap();
    let user_buffer = process.transfer_buffer(event_buf, len);
    let mut count = 0;
    for buf in user_buffer {
        let mut index = 0;
        let len = buf.len();
        while index < len {
            let event = read_event();
            if event == 0 {
                break;
            }
            buf[index] = event;
            index += 1;
            count += 1;
        }
    }
    count
}

fn read_event() -> u64 {
    let (keyboard, mouse) = {
        let kb = KEYBOARD_INPUT_DEVICE.get().unwrap();
        let mouse = MOUSE_INPUT_DEVICE.get().unwrap();
        (kb, mouse)
    };
    if !keyboard.is_empty() {
        keyboard.read_event_with_block()
    } else if !mouse.is_empty() {
        mouse.read_event_with_block()
    } else {
        0
    }
}
