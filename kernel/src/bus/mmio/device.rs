use alloc::collections::VecDeque;

use basic::{bus::mmio::*, io::SafeIORegion};
use mem::PhysAddr;

use crate::bus::CommonDeviceInfo;
pub struct MmioBus {
    common_devices: VecDeque<MmioCommonDevice>,
    // devices: Vec<Arc<dyn MmioDevice>>,
}

impl MmioBus {
    pub(super) const fn new() -> Self {
        Self {
            common_devices: VecDeque::new(),
            // devices: Vec::new(),
        }
    }
    pub(super) fn register_mmio_device(&mut self, device: MmioCommonDevice) {
        self.common_devices.push_back(device);
    }

    pub fn register_driver(&mut self) {
        // self.drivers.push(driver);
    }

    pub fn common_devices(&self) -> &VecDeque<MmioCommonDevice> {
        &self.common_devices
    }
}

#[derive(Debug, Clone)]
pub struct MmioCommonDevice {
    io_region: SafeIORegion,
    info: CommonDeviceInfo,
}

impl MmioCommonDevice {
    pub(super) fn new(io_region: SafeIORegion, info: CommonDeviceInfo) -> Self {
        let res = Self { io_region, info };
        info!(
            "[MmioCommonDevice]: Found Virtio mmio device, device type: {:?}, irq number: {:?}",
            res.device_type(),
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

    pub fn device_id(&self) -> u32 {
        self.io_region.read_at::<u32>(8).unwrap()
    }

    pub fn device_type(&self) -> VirtioMmioDeviceType {
        let id = self.device_id();
        VirtioMmioDeviceType::try_from(id as u8).unwrap()
    }

    pub fn version(&self) -> VirtioMmioVersion {
        VirtioMmioVersion::try_from(self.io_region.read_at::<u32>(4).unwrap()).unwrap()
    }

    pub fn irq(&self) -> Option<u32> {
        self.info.irq
    }
}
