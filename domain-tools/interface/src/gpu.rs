use constants::AlienResult;
use gproxy::proxy;
use rref::RRefVec;

use crate::{devices::DeviceInfo, Basic, DeviceBase};

#[proxy(GpuDomainProxy)]
pub trait GpuDomain: DeviceBase + Basic {
    fn init(&self, device_info: &DeviceInfo) -> AlienResult<()>;
    fn flush(&self) -> AlienResult<()>;
    fn fill(&self, offset: u32, buf: &RRefVec<u8>) -> AlienResult<usize>;
}
