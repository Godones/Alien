use alloc::string::{String, ToString};

use alloc::vec::Vec;
use core::ops::Range;
use fdt::standard_nodes::Compatible;
use fdt::Fdt;

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub name: String,
    pub base_addr: usize,
    pub irq: usize,
    pub compatible: String,
}

impl DeviceInfo {
    pub fn new(name: String, base_addr: usize, irq: usize, compatible: String) -> Self {
        Self {
            name,
            base_addr,
            irq,
            compatible,
        }
    }
}

pub trait Probe {
    /// Get the base address and irq number of the uart device from the device tree.
    fn probe_uart(&self) -> Option<DeviceInfo>;
    /// Get the base address and irq number of the rtc device from the device tree.
    fn probe_rtc(&self) -> Option<DeviceInfo>;
    /// Get the base address and irq number of the virtio devices from the device tree.
    fn probe_virtio(&self) -> Option<Vec<DeviceInfo>>;
    fn probe_common(&self, device_name: &str) -> Option<DeviceInfo>;
    #[cfg(feature = "vf2")]
    fn probe_sdio(&self) -> Option<DeviceInfo>;
}

impl Probe for Fdt<'_> {
    fn probe_uart(&self) -> Option<DeviceInfo> {
        match self.probe_common("uart") {
            Some(device_info) => Some(device_info),
            None => self.probe_common("serial"),
        }
    }

    fn probe_rtc(&self) -> Option<DeviceInfo> {
        self.probe_common("rtc")
    }

    fn probe_virtio(&self) -> Option<Vec<DeviceInfo>> {
        let mut virtio_devices = Vec::new();
        for node in self.all_nodes() {
            if node.name.starts_with("virtio_mmio") {
                let reg = node.reg()?.next()?;
                let paddr = reg.starting_address as usize;
                let irq = node.property("interrupts")?.value;
                let irq = u32::from_be_bytes(irq[0..4].try_into().ok()?);

                let compatible = node.compatible().map(Compatible::first).unwrap();

                virtio_devices.push(DeviceInfo::new(
                    String::from("virtio_mmio"),
                    paddr,
                    irq as usize,
                    compatible.to_string(),
                ));
            }
        }
        Some(virtio_devices)
    }

    fn probe_common(&self, device_name: &str) -> Option<DeviceInfo> {
        let node = self
            .all_nodes()
            .find(|node| node.name.starts_with(device_name))?;
        let reg = node.reg()?.next()?;
        let range = Range {
            start: reg.starting_address as usize,
            end: reg.starting_address as usize + reg.size.unwrap(),
        };
        let irq = node.property("interrupts").unwrap().value;
        let irq = u32::from_be_bytes(irq[0..4].try_into().unwrap());
        let compatible = node.compatible().map(Compatible::first).unwrap();
        Some(DeviceInfo::new(
            device_name.to_string(),
            range.start,
            irq as usize,
            compatible.to_string(),
        ))
    }

    #[cfg(feature = "vf2")]
    fn probe_sdio(&self) -> Option<DeviceInfo> {
        self.probe_common("sdio1")
    }
}
