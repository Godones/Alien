use constants::AlienResult;
use rref::RRefVec;

use crate::DeviceBase;

pub trait CacheBlkDeviceDomain: DeviceBase {
    fn init(&self, blk_domain_name: &str) -> AlienResult<()>;
    fn read(&self, offset: u64, buf: RRefVec<u8>) -> AlienResult<RRefVec<u8>>;
    fn write(&self, offset: u64, buf: &RRefVec<u8>) -> AlienResult<usize>;
    fn get_capacity(&self) -> AlienResult<u64>;
    fn flush(&self) -> AlienResult<()>;
}
