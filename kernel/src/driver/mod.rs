pub use block_device::{QEMU_BLOCK_DEVICE, QemuBlockDevice};
pub use dtb::{DEVICE_TABLE, init_dt, PLIC};
pub use input::sys_event_get;
pub use mpci::pci_probe;

mod block_device;
mod dtb;
mod hal;
mod mpci;

pub mod rtc;
pub mod uart;
pub mod gpu;
pub mod input;

pub trait DeviceBase: Sync + Send {
    fn hand_irq(&self);
}
