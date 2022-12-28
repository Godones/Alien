mod block_device;
mod dtb;
mod hal;

pub use block_device::{QemuBlockDevice, QEMU_BLOCK_DEVICE};
pub use dtb::init_dt;
pub trait Device {
    fn init(&self);
}
