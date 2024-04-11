use constants::{io::RtcTime, AlienResult};
use gproxy::proxy;
use rref::RRef;

use crate::{devices::DeviceInfo, Basic, DeviceBase};

#[proxy(RtcDomainProxy)]
pub trait RtcDomain: DeviceBase + Basic {
    fn init(&self, device_info: &DeviceInfo) -> AlienResult<()>;
    fn read_time(&self, time: RRef<RtcTime>) -> AlienResult<RRef<RtcTime>>;
}
