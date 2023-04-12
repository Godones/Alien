mod block_device;
mod dtb;
mod hal;
mod mpci;

pub use block_device::{QemuBlockDevice, QEMU_BLOCK_DEVICE};
pub use dtb::{init_dt, DEVICE_TABLE, PLIC};
pub use mpci::pci_probe;
pub mod rtc;
pub mod uart;
pub mod uart1;

pub trait DeviceBase: Sync + Send {
    fn hand_irq(&self);
}
