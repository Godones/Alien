use alloc::boxed::Box;

use constants::{AlienError, AlienResult};
use interface::{Basic, EmptyDeviceDomain};
use rref::RRefVec;

#[derive(Debug)]
pub struct EmptyDeviceDomainProxy {
    id: u64,
    domain: Box<dyn EmptyDeviceDomain>,
}

impl EmptyDeviceDomainProxy {
    pub fn new(id: u64, domain: Box<dyn EmptyDeviceDomain>) -> Self {
        Self { id, domain }
    }
}

impl Basic for EmptyDeviceDomainProxy {
    fn is_active(&self) -> bool {
        self.domain.is_active()
    }
}

impl EmptyDeviceDomain for EmptyDeviceDomainProxy {
    fn init(&self) -> AlienResult<()> {
        self.domain.init()
    }

    fn read(&self, data: RRefVec<u8>) -> AlienResult<RRefVec<u8>> {
        if self.is_active() {
            self.domain.read(data)
        } else {
            Err(AlienError::DOMAINCRASH)
        }
    }

    fn write(&self, data: &RRefVec<u8>) -> AlienResult<usize> {
        if self.is_active() {
            self.domain.write(data)
        } else {
            Err(AlienError::DOMAINCRASH)
        }
    }
}
