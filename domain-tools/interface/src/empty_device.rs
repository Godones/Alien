use constants::AlienResult;
use rref::RRefVec;

use crate::Basic;

pub trait EmptyDeviceDomain: Basic {
    fn init(&self) -> AlienResult<()>;
    fn read(&self, data: RRefVec<u8>) -> AlienResult<RRefVec<u8>>;
    fn write(&self, data: &RRefVec<u8>) -> AlienResult<usize>;
}
