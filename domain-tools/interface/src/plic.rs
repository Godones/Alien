use crate::{Basic, DeviceBase, DeviceInfo};
use alloc::sync::Arc;
use constants::AlienResult;
use rref::RRefVec;

pub trait PLICDomain: Basic {
    fn init(&self, device_info: &DeviceInfo) -> AlienResult<()>;
    fn handle_irq(&self) -> AlienResult<()>;
    fn register_irq(&self, irq: usize, device: Arc<dyn DeviceBase>) -> AlienResult<()>;
    fn irq_info(&self, buf: RRefVec<u8>) -> AlienResult<RRefVec<u8>>;
}
