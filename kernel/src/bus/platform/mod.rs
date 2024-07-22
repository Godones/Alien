use alloc::string::ToString;

use basic::io::SafeIORegion;
use device::{PlatformBus, PlatformCommonDevice};
use ksync::Mutex;

use crate::bus::CommonDeviceInfo;

mod device;

pub static PLATFORM_BUS: Mutex<PlatformBus> = Mutex::new(PlatformBus::new());

pub fn register_platform_device(info: CommonDeviceInfo, name: &str) {
    let io_region = SafeIORegion::new(info.address_range.clone());
    let platform_device = PlatformCommonDevice::new(io_region, info, name.to_string());

    PLATFORM_BUS.lock().register_common_device(platform_device);
}

pub fn register_platform_driver() {
    // PLATFORM_BUS.lock().register_driver(driver);
}

#[macro_export]
macro_rules! platform_bus {
    () => {
        $crate::bus::platform::PLATFORM_BUS.lock()
    };
}
