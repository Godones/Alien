use alloc::string::ToString;

use crate::{SystemDeviceInfo, SystemDeviceType};
use alloc::vec::Vec;
use basic::println;
use core::ops::Range;
use core::ptr::NonNull;
use fdt::standard_nodes::Compatible;
use fdt::Fdt;
use log::info;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
use virtio_drivers::transport::{DeviceType, Transport};

pub trait Probe {
    /// Get the base address and irq number of the uart device from the device tree.
    fn probe_uart(&self) -> Option<SystemDeviceType>;
    /// Get the base address and irq number of the rtc device from the device tree.
    fn probe_rtc(&self) -> Option<SystemDeviceType>;
    fn probe_plic(&self) -> Option<SystemDeviceType>;
    /// Get the base address and irq number of the virtio devices from the device tree.
    fn probe_virtio(&self) -> Option<Vec<SystemDeviceType>>;
    fn probe_common(&self, device_name: &str) -> Option<SystemDeviceInfo>;
}

impl Probe for Fdt<'_> {
    fn probe_uart(&self) -> Option<SystemDeviceType> {
        match self.probe_common("uart") {
            Some(device_info) => Some(SystemDeviceType::Uart(device_info)),
            None => match self.probe_common("serial") {
                Some(device_info) => Some(SystemDeviceType::Uart(device_info)),
                None => None,
            },
        }
    }

    fn probe_rtc(&self) -> Option<SystemDeviceType> {
        match self.probe_common("rtc") {
            Some(device_info) => Some(SystemDeviceType::Rtc(device_info)),
            None => None,
        }
    }

    fn probe_plic(&self) -> Option<SystemDeviceType> {
        let node = self
            .all_nodes()
            .find(|node| node.name.starts_with("plic"))?;
        let reg = node.reg()?.next()?;
        let range = Range {
            start: reg.starting_address as usize,
            end: reg.starting_address as usize + reg.size.unwrap(),
        };
        let compatible = node.compatible().map(Compatible::first).unwrap();
        let device_info = SystemDeviceInfo {
            address_range: range,
            irq: None,
            compatible: Some(compatible.to_string()),
        };
        Some(SystemDeviceType::PLIC(device_info))
    }

    fn probe_virtio(&self) -> Option<Vec<SystemDeviceType>> {
        let mut virtio_devices = Vec::new();
        for node in self.all_nodes() {
            if node.name.starts_with("virtio_mmio") {
                let reg = node.reg().unwrap().next().unwrap();
                let range = Range {
                    start: reg.starting_address as usize,
                    end: reg.starting_address as usize + reg.size.unwrap(),
                };
                let irq = node.property("interrupts").unwrap().value;
                let irq = u32::from_be_bytes(irq[0..4].try_into().ok().unwrap());

                let compatible = node.compatible().map(Compatible::first).unwrap();

                // info!("range: {:#x?}",range);

                let header = NonNull::new(range.start as *mut VirtIOHeader).unwrap();
                info!("range: {:#x?}", range);
                let device_info = SystemDeviceInfo {
                    address_range: range,
                    irq: Some(irq),
                    compatible: Some(compatible.to_string()),
                };

                unsafe {
                    match MmioTransport::new(header) {
                        Err(e) => {
                            info!("mmio error is :{}", e)
                        }
                        Ok(mut transport) => {
                            info!(
                                "Detected virtio MMIO device with vendor id {:#X}, device type {:?}, version {:?}, features:{:?}",
                                transport.vendor_id(),
                                transport.device_type(),
                                transport.version(),
                                transport.read_device_features(),
                            );
                            match transport.device_type() {
                                DeviceType::Input => virtio_devices
                                    .push(SystemDeviceType::VirtIoMMioInput(device_info)),
                                DeviceType::Block => virtio_devices
                                    .push(SystemDeviceType::VirtIoMMioBlock(device_info)),
                                DeviceType::GPU => virtio_devices
                                    .push(SystemDeviceType::VirtIoMMioGpu(device_info)),
                                DeviceType::Network => virtio_devices
                                    .push(SystemDeviceType::VirtIoMMioNet(device_info)),
                                ty => {
                                    println!("Don't support virtio device type: {:?}", ty);
                                }
                            }
                        }
                    }
                }
            }
        }
        Some(virtio_devices)
    }

    fn probe_common(&self, device_name: &str) -> Option<SystemDeviceInfo> {
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

        let device_info = SystemDeviceInfo {
            address_range: range,
            irq: Some(irq),
            compatible: Some(compatible.to_string()),
        };

        Some(device_info)
    }
}
