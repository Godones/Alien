use alloc::collections::VecDeque;

use basic::io::SafeIORegion;

use crate::bus::CommonDeviceInfo;

pub struct PciBus {
    common_devices: VecDeque<PciCommonDevice>,
}
#[derive(Debug)]
pub struct PciCommonDevice {
    io_region: SafeIORegion,
    info: CommonDeviceInfo,
}

impl PciBus {
    pub(super) const fn new() -> Self {
        Self {
            common_devices: VecDeque::new(),
        }
    }
    pub(super) fn register_common_device(&mut self, device: PciCommonDevice) {
        self.common_devices.push_back(device);
    }

    pub fn register_driver(&mut self) {
        // self.drivers.push(driver);
    }
}
