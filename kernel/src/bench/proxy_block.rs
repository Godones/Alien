use alloc::boxed::Box;
use core::{any::Any, ops::Range};

use interface::*;
use jtable::*;
use ksync::RwLock;
use paste::paste;
use rref::{RRefVec, SharedData};
use spin::Once;

use crate::{
    domain_loader::loader::DomainLoader,
    domain_proxy::{PerCpuCounter, ProxyBuilder},
    error::{AlienError, AlienResult},
    sync::{RcuData, SleepMutex},
};

define_static_key_false!(BLKDOMAINPROXY_KEY_FAKE);

#[derive(Debug)]
pub struct BlkDomainProxy {
    domain: RcuData<Box<dyn BlkDeviceDomain>>,
    lock: RwLock<()>,
    domain_loader: SleepMutex<DomainLoader>,
    counter: PerCpuCounter,
    resource: Once<Box<dyn Any + Send + Sync>>,
}
#[allow(unused)]
impl BlkDomainProxy {
    pub fn new(domain: Box<dyn BlkDeviceDomain>, domain_loader: DomainLoader) -> Self {
        Self {
            domain: RcuData::new(Box::new(domain)),
            lock: RwLock::new(()),
            domain_loader: SleepMutex::new(domain_loader),
            counter: PerCpuCounter::new(),
            resource: Once::new(),
        }
    }
    pub fn all_counter(&self) -> usize {
        self.counter.all()
    }
    pub fn domain_loader(&self) -> DomainLoader {
        self.domain_loader.lock().clone()
    }
}
impl ProxyBuilder for BlkDomainProxy {
    type T = Box<dyn BlkDeviceDomain>;
    fn build(domain: Self::T, domain_loader: DomainLoader) -> Self {
        Self::new(domain, domain_loader)
    }
    fn build_empty(_domain_loader: DomainLoader) -> Self {
        todo!()
    }
    fn init_by_box(&self, argv: Box<dyn Any + Send + Sync>) -> AlienResult<()> {
        let arg = argv.as_ref().downcast_ref::<Range<usize>>().unwrap();
        self.init(arg)?;
        self.resource.call_once(|| argv);
        Ok(())
    }
}
impl BlkDomainProxy {
    #[inline(always)]
    fn __handle_irq(&self) -> AlienResult<()> {
        let domain = self.domain.get();
        if !domain.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        domain.handle_irq()
    }
    #[inline(always)]
    fn __handle_irq_no_lock(&self) -> AlienResult<()> {
        self.counter.inc();
        let res = self.__handle_irq();
        self.counter.dec();
        res
    }
    #[cold]
    #[inline(always)]
    fn __handle_irq_with_lock(&self) -> AlienResult<()> {
        let r_lock = self.lock.read();
        let res = self.__handle_irq();
        drop(r_lock);
        res
    }
}
impl DeviceBase for BlkDomainProxy {
    fn handle_irq(&self) -> AlienResult<()> {
        if static_branch_likely!(BLKDOMAINPROXY_KEY_FAKE) {
            return self.__handle_irq_with_lock();
        }
        self.__handle_irq_no_lock()
    }
}
impl Basic for BlkDomainProxy {
    fn domain_id(&self) -> u64 {
        self.domain.get().domain_id()
    }
    fn is_active(&self) -> bool {
        self.domain.get().is_active()
    }
}
impl BlkDeviceDomain for BlkDomainProxy {
    fn init(&self, device_info: &Range<usize>) -> AlienResult<()> {
        self.domain.get().init(device_info)
    }
    fn read_block(&self, block: u32, data: RRefVec<u8>) -> AlienResult<RRefVec<u8>> {
        if static_branch_likely!(BLKDOMAINPROXY_KEY_FAKE) {
            return self.__read_block_with_lock(block, data);
        }
        self.__read_block_no_lock(block, data)
    }
    fn write_block(&self, block: u32, data: &RRefVec<u8>) -> AlienResult<usize> {
        if static_branch_likely!(BLKDOMAINPROXY_KEY_FAKE) {
            return self.__write_block_with_lock(block, data);
        }
        self.__write_block_no_lock(block, data)
    }
    fn get_capacity(&self) -> AlienResult<u64> {
        if static_branch_likely!(BLKDOMAINPROXY_KEY_FAKE) {
            return self.__get_capacity_with_lock();
        }
        self.__get_capacity_no_lock()
    }
    fn flush(&self) -> AlienResult<()> {
        if static_branch_likely!(BLKDOMAINPROXY_KEY_FAKE) {
            return self.__flush_with_lock();
        }
        self.__flush_no_lock()
    }
}
impl BlkDomainProxy {
    #[inline(always)]
    fn __read_block(&self, block: u32, data: RRefVec<u8>) -> AlienResult<RRefVec<u8>> {
        let r_domain = self.domain.get();
        let id = r_domain.domain_id();
        let old_id = data.move_to(id);
        let res = r_domain.read_block(block, data).map(|r| {
            r.move_to(old_id);
            r
        });
        res
    }
    #[inline(always)]
    fn __read_block_no_lock(&self, block: u32, data: RRefVec<u8>) -> AlienResult<RRefVec<u8>> {
        self.counter.inc();
        let res = self.__read_block(block, data);
        self.counter.dec();
        res
    }
    #[cold]
    #[inline(always)]
    fn __read_block_with_lock(&self, block: u32, data: RRefVec<u8>) -> AlienResult<RRefVec<u8>> {
        let r_lock = self.lock.read();
        let res = self.__read_block(block, data);
        drop(r_lock);
        res
    }
    #[inline(always)]
    fn __write_block(&self, block: u32, data: &RRefVec<u8>) -> AlienResult<usize> {
        let r_domain = self.domain.get();
        let res = r_domain.write_block(block, data).map(|r| r);
        res
    }
    #[inline(always)]
    fn __write_block_no_lock(&self, block: u32, data: &RRefVec<u8>) -> AlienResult<usize> {
        self.counter.inc();
        let res = self.__write_block(block, data);
        self.counter.dec();
        res
    }
    #[cold]
    #[inline(always)]
    fn __write_block_with_lock(&self, block: u32, data: &RRefVec<u8>) -> AlienResult<usize> {
        let r_lock = self.lock.read();
        let res = self.__write_block(block, data);
        drop(r_lock);
        res
    }
    #[inline(always)]
    fn __get_capacity(&self) -> AlienResult<u64> {
        let r_domain = self.domain.get();
        let res = r_domain.get_capacity().map(|r| r);
        res
    }
    #[inline(always)]
    fn __get_capacity_no_lock(&self) -> AlienResult<u64> {
        self.counter.inc();
        let res = self.__get_capacity();
        self.counter.dec();
        res
    }
    #[cold]
    #[inline(always)]
    fn __get_capacity_with_lock(&self) -> AlienResult<u64> {
        let r_lock = self.lock.read();
        let res = self.__get_capacity();
        drop(r_lock);
        res
    }
    #[inline(always)]
    fn __flush(&self) -> AlienResult<()> {
        let r_domain = self.domain.get();
        let res = r_domain.flush().map(|r| r);
        res
    }
    #[inline(always)]
    fn __flush_no_lock(&self) -> AlienResult<()> {
        self.counter.inc();
        let res = self.__flush();
        self.counter.dec();
        res
    }
    #[cold]
    #[inline(always)]
    fn __flush_with_lock(&self) -> AlienResult<()> {
        let r_lock = self.lock.read();
        let res = self.__flush();
        drop(r_lock);
        res
    }
}
