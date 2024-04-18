use constants::AlienResult;
use downcast_rs::{impl_downcast, DowncastSync};
use gproxy::proxy;
use rref::RRefVec;

use crate::Basic;

#[proxy(EmptyDeviceDomainProxy)]
pub trait EmptyDeviceDomain: Basic + DowncastSync {
    fn init(&self) -> AlienResult<()>;
    fn read(&self, data: RRefVec<u8>) -> AlienResult<RRefVec<u8>>;
    fn write(&self, data: &RRefVec<u8>) -> AlienResult<usize>;
}

impl_downcast!(sync EmptyDeviceDomain);
