use constants::AlienResult;
use gproxy::proxy;
use rref::RRefVec;

use crate::Basic;

#[proxy(EmptyDeviceDomainProxy)]
pub trait EmptyDeviceDomain: Basic {
    fn init(&self) -> AlienResult<()>;
    fn read(&self, data: RRefVec<u8>) -> AlienResult<RRefVec<u8>>;
    fn write(&self, data: &RRefVec<u8>) -> AlienResult<usize>;
}
