use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};

use constants::{AlienError, AlienResult};
use interface::{Basic, DeviceBase, ShadowBlockDomain};
use ksync::RwLock;
use rref::RRef;
use spin::Mutex;

#[derive(Debug)]
pub struct ShadowBlockDomainProxy {
    id: u64,
    domain: RwLock<Box<dyn ShadowBlockDomain>>,
    old: Mutex<Vec<Box<dyn ShadowBlockDomain>>>,
    backend_domain: Mutex<String>,
}
impl ShadowBlockDomainProxy {
    pub fn new(id: u64, domain: Box<dyn ShadowBlockDomain>) -> Self {
        Self {
            id,
            domain: RwLock::new(domain),
            old: Mutex::new(Vec::new()),
            backend_domain: Mutex::new(String::new()),
        }
    }
    pub fn replace(&self, domain: Box<dyn ShadowBlockDomain>) {
        let mut old_domain = self.domain.write();
        let mut old = self.old.lock();
        // swap the old domain with the new one
        // and push the old domain to the old domain list( we will fix it)
        old.push(core::mem::replace(&mut *old_domain, domain));
    }
    pub fn backend_domain(&self) -> String {
        self.backend_domain.lock().clone()
    }
}
impl DeviceBase for ShadowBlockDomainProxy {
    fn handle_irq(&self) -> AlienResult<()> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.read().handle_irq()
    }
}
impl Basic for ShadowBlockDomainProxy {
    fn is_active(&self) -> bool {
        self.domain.read().is_active()
    }
}
impl ShadowBlockDomain for ShadowBlockDomainProxy {
    fn init(&self, blk_domain: &str) -> AlienResult<()> {
        *self.backend_domain.lock() = blk_domain.to_string();
        self.domain.read().init(blk_domain)
    }
    fn read_block(&self, block: u32, data: RRef<[u8; 512]>) -> AlienResult<RRef<[u8; 512]>> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.read().read_block(block, data)
    }
    fn write_block(&self, block: u32, data: &RRef<[u8; 512]>) -> AlienResult<usize> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.read().write_block(block, data)
    }
    fn get_capacity(&self) -> AlienResult<u64> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.read().get_capacity()
    }
    fn flush(&self) -> AlienResult<()> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.read().flush()
    }
}
