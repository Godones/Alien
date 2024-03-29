use alloc::boxed::Box;
use alloc::sync::Arc;

use constants::{AlienError, AlienResult};
use interface::{Basic, DeviceBase, DeviceInfo, InputDomain};
use rref::RRefVec;

#[derive(Debug)]
pub struct InputDomainProxy {
    id: u64,
    domain: Box<dyn InputDomain>,
}

impl InputDomainProxy {
    pub fn new(id: u64, domain: Box<dyn InputDomain>) -> Self {
        Self { id, domain }
    }
}

impl InputDomain for InputDomainProxy {
    fn init(&self, device_info: &DeviceInfo) -> AlienResult<()> {
        self.domain.init(device_info)
    }

    fn event_nonblock(&self) -> AlienResult<Option<u64>> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.event_nonblock()
    }
}

impl DeviceBase for InputDomainProxy {
    fn handle_irq(&self) -> AlienResult<()> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.handle_irq()
    }
}

impl Basic for InputDomainProxy {
    fn is_active(&self) -> bool {
        self.domain.is_active()
    }
}
