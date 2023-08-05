use alloc::sync::Arc;

use rvfs::superblock::Device;
use spin::Once;

use crate::interrupt::DeviceBase;

pub trait BlockDevice: Device + DeviceBase {}

pub static BLOCK_DEVICE: Once<Arc<dyn Device>> = Once::new();

pub fn init_block_device(block_device: Arc<dyn Device>) {
    // BLOCK_DEVICE.lock().push(block_device);
    BLOCK_DEVICE.call_once(|| block_device);
}
