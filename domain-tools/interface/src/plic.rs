use constants::AlienResult;
use gproxy::proxy;
use rref::RRefVec;

use crate::{devices::DeviceInfo, Basic};
#[proxy(PLICDomainProxy)]
pub trait PLICDomain: Basic {
    fn init(&self, device_info: &DeviceInfo) -> AlienResult<()>;
    fn handle_irq(&self) -> AlienResult<()>;
    fn register_irq(&self, irq: usize, device_domain_name: &RRefVec<u8>) -> AlienResult<()>;
    fn irq_info(&self, buf: RRefVec<u8>) -> AlienResult<RRefVec<u8>>;
}
