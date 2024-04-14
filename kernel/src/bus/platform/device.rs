use alloc::{collections::VecDeque, string::String};

use basic::io::SafeIORegion;
use mem::PhysAddr;

use crate::bus::CommonDeviceInfo;

pub struct PlatformBus {
    common_devices: VecDeque<PlatformCommonDevice>,
}

impl PlatformBus {
    pub(super) const fn new() -> Self {
        Self {
            common_devices: VecDeque::new(),
        }
    }
    pub(super) fn register_common_device(&mut self, device: PlatformCommonDevice) {
        self.common_devices.push_back(device);
    }

    pub fn register_driver(&mut self) {
        // self.drivers.push(driver);
    }

    pub fn common_devices(&self) -> &VecDeque<PlatformCommonDevice> {
        &self.common_devices
    }
}
#[derive(Debug)]
pub struct PlatformCommonDevice {
    io_region: SafeIORegion,
    info: CommonDeviceInfo,
    name: String,
}

impl PlatformCommonDevice {
    pub(super) fn new(io_region: SafeIORegion, info: CommonDeviceInfo, name: String) -> Self {
        let res = Self {
            io_region,
            info,
            name,
        };
        info!(
            "[PlatformCommonDevice]: Found platform device, device name:{:?}, irq number:{:?}",
            res.name(),
            res.info.irq
        );
        res
    }

    pub fn address(&self) -> PhysAddr {
        self.io_region.phys_addr()
    }

    pub fn io_region(&self) -> &SafeIORegion {
        &self.io_region
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn irq(&self) -> Option<u32> {
        self.info.irq
    }
}
