pub mod continuation;

use alloc::{borrow::ToOwned, boxed::Box, string::String, vec::Vec};
use core::{net::SocketAddrV4, ops::Range, sync::atomic::AtomicU64};

use downcast_rs::{impl_downcast, DowncastSync};
use interface::*;
use ksync::{Mutex, RwLock};
use pconst::{
    io::{PollEvents, RtcTime, SeekFrom},
    net::*,
};
use rref::{RRef, RRefVec};
use spin::Once;
use task_meta::TaskSchedulingInfo;
use vfscore::{fstype::FileSystemFlags, inode::InodeAttr, superblock::SuperType, utils::*};

use crate::{
    domain_loader::loader::DomainLoader,
    error::{AlienError, AlienResult},
    sync::{RcuData, SRcuLock},
};
gen_for_BufInputDomain!();
gen_for_BufUartDomain!();
gen_for_CacheBlkDeviceDomain!();
gen_for_EmptyDeviceDomain!();
gen_for_FsDomain!();
gen_for_GpuDomain!();
gen_for_InputDomain!();
gen_for_NetDeviceDomain!();
gen_for_RtcDomain!();
gen_for_SysCallDomain!();
gen_for_TaskDomain!();
gen_for_UartDomain!();
gen_for_VfsDomain!();
gen_for_PLICDomain!();
gen_for_SchedulerDomain!();
gen_for_ShadowBlockDomain!();
gen_for_BlkDeviceDomain!();

gen_for_DevFsDomain!();
// gen_for_LogDomain!();
gen_for_NetDomain!();
impl_for_FsDomain!(DevFsDomainProxy);
impl Basic for DevFsDomainProxy {
    fn is_active(&self) -> bool {
        self.domain.read().is_active()
    }
}

pub trait ProxyExt: DowncastSync {
    fn reload(&self) -> AlienResult<()>;
}

impl_downcast!(sync ProxyExt);

impl ShadowBlockDomainProxy {
    pub fn replace(
        &self,
        new_domain: Box<dyn ShadowBlockDomain>,
        loader: DomainLoader,
    ) -> AlienResult<()> {
        let mut old_domain = self.domain.write();
        let mut old = self.old_domain.lock();
        *self.domain_loader.lock() = loader;
        // swap the old domain with the new one
        // and push the old domain to the old domain list( we will fix it)
        old.push(core::mem::replace(&mut *old_domain, new_domain));
        let resource = self.resource.get().unwrap();
        old_domain.init(resource.as_str())
    }
}

impl ProxyExt for BlkDomainProxy {
    fn reload(&self) -> AlienResult<()> {
        let mut domain = self.domain.write();
        self.domain_loader.lock().reload().unwrap();
        // let mut loader = DomainLoader::new(self.domain_loader.data());
        // loader.load().unwrap();
        // let new_domain = loader.call(self.domain_id);
        let id = self.id.load(core::sync::atomic::Ordering::Relaxed);
        let mut new_domain = self.domain_loader.lock().call(id);
        let device_info = self.resource.get().unwrap();
        // new_domain.init(device_info.clone()).unwrap();
        core::mem::swap(&mut *domain, &mut new_domain);
        // The new_domain now is the old domain, but it has been recycled so we
        // can't drop it again
        domain.init(device_info.clone()).unwrap();
        core::mem::forget(new_domain);
        Ok(())
    }
}

impl SchedulerDomainProxy {
    pub fn replace(
        &self,
        _new_domain: Box<dyn SchedulerDomain>,
        _loader: DomainLoader,
    ) -> AlienResult<()> {
        let old_domain = self.domain.write();
        // let mut old = self.old_domain.lock();
        // *self.domain_loader.lock() = loader;
        // // swap the old domain with the new one
        // // and push the old domain to the old domain list( we will fix it)
        // old.push(core::mem::replace(&mut *old_domain, new_domain));
        // old_domain.init()
        println!("Try dump old domain data");
        let mut data = SchedulerDataContainer::default();
        old_domain.dump_meta_data(&mut data)?;
        println!("old domain data: {:?}", data);
        Err(AlienError::EINVAL)
    }
}

#[derive(Debug)]
pub struct LogDomainProxy {
    id: AtomicU64,
    domain: RcuData<Box<dyn LogDomain>>,
    srcu_lock: SRcuLock,
    old_domain: Mutex<Vec<Box<Box<dyn LogDomain>>>>,
    domain_loader: Mutex<DomainLoader>,
}
impl LogDomainProxy {
    pub fn new(id: u64, domain: Box<dyn LogDomain>, domain_loader: DomainLoader) -> Self {
        Self {
            id: AtomicU64::new(id),
            domain: RcuData::new(Box::new(domain)),
            srcu_lock: SRcuLock::new(),
            old_domain: Mutex::new(Vec::new()),
            domain_loader: Mutex::new(domain_loader),
        }
    }
}
impl Basic for LogDomainProxy {
    fn is_active(&self) -> bool {
        let idx = self.srcu_lock.read_lock();
        let res = self.domain.get().is_active();
        self.srcu_lock.read_unlock(idx);
        res
    }
}
impl LogDomain for LogDomainProxy {
    fn init(&self) -> AlienResult<()> {
        let idx = self.srcu_lock.read_lock();
        let res = self.domain.get().init();
        self.srcu_lock.read_unlock(idx);
        res
    }
    fn log(&self, level: Level, msg: RRefVec<u8>) -> AlienResult<()> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        let idx = self.srcu_lock.read_lock();
        let res = self.domain.get().log(level, msg);
        self.srcu_lock.read_unlock(idx);
        res
    }
    fn set_max_level(&self, level: LevelFilter) -> AlienResult<()> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        let idx = self.srcu_lock.read_lock();
        let res = self.domain.get().set_max_level(level);
        self.srcu_lock.read_unlock(idx);
        res
    }
}

impl LogDomainProxy {
    pub fn replace(&self, new_domain: Box<dyn LogDomain>, loader: DomainLoader) -> AlienResult<()> {
        // let mut old_domain = self.domain.write();
        let mut old = self.old_domain.lock();
        *self.domain_loader.lock() = loader;

        let old_domain = self.domain.swap(Box::new(new_domain));

        // synchronize the reader which is reading the old domain
        self.srcu_lock.synchronize();

        // recycle the old domain
        old.push(old_domain.into());

        let idx = self.srcu_lock.read_lock();
        let res = self.domain.get().init();
        self.srcu_lock.read_unlock(idx);
        res
    }
}
