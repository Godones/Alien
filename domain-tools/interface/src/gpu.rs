use core::ops::Range;

use constants::AlienResult;
use downcast_rs::{impl_downcast, DowncastSync};
use gproxy::proxy;
use rref::RRefVec;

use crate::{Basic, DeviceBase};

#[proxy(GpuDomainProxy,Range<usize>)]
pub trait GpuDomain: DeviceBase + Basic + DowncastSync {
    fn init(&self, device_info: Range<usize>) -> AlienResult<()>;
    fn flush(&self) -> AlienResult<()>;
    fn fill(&self, offset: u32, buf: &RRefVec<u8>) -> AlienResult<usize>;
}

impl_downcast!(sync GpuDomain);
