mod block_device;
mod dtb;
mod hal;
mod mpci;

pub use block_device::{QemuBlockDevice, QEMU_BLOCK_DEVICE};
pub use dtb::init_dt;
pub use mpci::pci_probe;
pub mod rtc;

pub trait Device {
    fn init(&self);
}
