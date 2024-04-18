use core::ops::Range;

use constants::{io::RtcTime, AlienResult};
use downcast_rs::{impl_downcast, DowncastSync};
use gproxy::proxy;
use rref::RRef;

use crate::{Basic, DeviceBase};

#[proxy(RtcDomainProxy,Range<usize>)]
pub trait RtcDomain: DeviceBase + Basic + DowncastSync {
    fn init(&self, device_info: Range<usize>) -> AlienResult<()>;
    fn read_time(&self, time: RRef<RtcTime>) -> AlienResult<RRef<RtcTime>>;
}

impl_downcast!(sync RtcDomain);
