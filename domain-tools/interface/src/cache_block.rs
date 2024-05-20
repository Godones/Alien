use downcast_rs::{impl_downcast, DowncastSync};
use gproxy::proxy;
use rref::RRefVec;

use super::AlienResult;
use crate::{Basic, DeviceBase};

#[proxy(CacheBlkDomainProxy, String)]
pub trait CacheBlkDeviceDomain: DeviceBase + Basic + DowncastSync {
    fn init(&self, blk_domain_name: &str) -> AlienResult<()>;
    fn read(&self, offset: u64, buf: RRefVec<u8>) -> AlienResult<RRefVec<u8>>;
    fn write(&self, offset: u64, buf: &RRefVec<u8>) -> AlienResult<usize>;
    fn get_capacity(&self) -> AlienResult<u64>;
    fn flush(&self) -> AlienResult<()>;
}

impl_downcast!(sync CacheBlkDeviceDomain);
