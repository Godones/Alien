use crate::DeviceBase;
use rref::{RRefVec, RpcResult};

pub trait CacheBlkDeviceDomain: DeviceBase {
    fn read(&self, offset: u64, buf: RRefVec<u8>) -> RpcResult<RRefVec<u8>>;
    fn write(&self, offset: u64, buf: &RRefVec<u8>) -> RpcResult<usize>;
    fn get_capacity(&self) -> RpcResult<u64>;
    fn flush(&self) -> RpcResult<()>;
}
