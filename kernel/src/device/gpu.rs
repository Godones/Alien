use alloc::sync::Arc;
use core::any::Any;

use spin::Once;

use crate::interrupt::DeviceBase;

pub trait GpuDevice: Send + Sync + Any + DeviceBase {
    fn update_cursor(&self);
    fn get_framebuffer(&self) -> &mut [u8];
    fn flush(&self);
}

pub static GPU_DEVICE: Once<Arc<dyn GpuDevice>> = Once::new();

#[allow(unused)]
pub fn init_gpu(gpu: Arc<dyn GpuDevice>) {
    GPU_DEVICE.call_once(|| gpu);
}
