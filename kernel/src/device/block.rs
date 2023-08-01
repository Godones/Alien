use alloc::sync::Arc;

use lazy_static::lazy_static;
use rvfs::superblock::Device;
use spin::Once;

use crate::interrupt::DeviceBase;

pub trait BlockDevice: Device + DeviceBase {}

// pub static BLOCK_DEVICE: Mutex<Vec<Arc<dyn BlockDevice>>> = Mutex::new(Vec::new());

lazy_static! {
    pub static ref BLOCK_DEVICE: Once<Arc<dyn Device>> = Once::new();
}

pub fn init_block_device(block_device: Arc<dyn Device>) {
    // BLOCK_DEVICE.lock().push(block_device);
    BLOCK_DEVICE.call_once(|| block_device);
}
