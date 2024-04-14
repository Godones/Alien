use core::ops::Range;

use constants::AlienResult;
use gproxy::proxy;
use rref::RRefVec;

use crate::{Basic, DeviceBase};

#[proxy(GpuDomainProxy)]
pub trait GpuDomain: DeviceBase + Basic {
    fn init(&self, device_info: Range<usize>) -> AlienResult<()>;
    fn flush(&self) -> AlienResult<()>;
    fn fill(&self, offset: u32, buf: &RRefVec<u8>) -> AlienResult<usize>;
}
