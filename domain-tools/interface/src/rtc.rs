use core::ops::Range;

use constants::{io::RtcTime, AlienResult};
use gproxy::proxy;
use rref::RRef;

use crate::{Basic, DeviceBase};

#[proxy(RtcDomainProxy)]
pub trait RtcDomain: DeviceBase + Basic {
    fn init(&self, device_info: Range<usize>) -> AlienResult<()>;
    fn read_time(&self, time: RRef<RtcTime>) -> AlienResult<RRef<RtcTime>>;
}
