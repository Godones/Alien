use crate::{Basic, DeviceBase};
use alloc::sync::Arc;
use rref::{RRefVec, RpcResult};

pub trait PLICDomain: Basic {
    fn handle_irq(&self) -> RpcResult<()>;
    fn register_irq(&self, irq: usize, device: Arc<dyn DeviceBase>) -> RpcResult<()>;
    fn irq_info(&self, buf: RRefVec<u8>) -> RpcResult<RRefVec<u8>>;
}
