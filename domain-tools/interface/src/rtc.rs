use constants::{io::RtcTime, AlienResult};
use rref::RRef;

use crate::{devices::DeviceInfo, DeviceBase};

pub trait RtcDomain: DeviceBase {
    fn init(&self, device_info: &DeviceInfo) -> AlienResult<()>;
    fn read_time(&self, time: RRef<RtcTime>) -> AlienResult<RRef<RtcTime>>;
}
