use crate::DeviceBase;
use rref::{RRefVec, RpcResult};

pub trait GpuDomain: DeviceBase {
    fn flush(&self) -> RpcResult<()>;
    fn fill(&self, offset: u32, buf: &RRefVec<u8>) -> RpcResult<usize>;
}
