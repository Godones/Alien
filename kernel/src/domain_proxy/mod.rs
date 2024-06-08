use alloc::{borrow::ToOwned, boxed::Box, string::String, vec::Vec};
use core::{fmt::Debug, mem::forget, net::SocketAddrV4, ops::Range, sync::atomic::AtomicU64};

use downcast_rs::{impl_downcast, DowncastSync};
use interface::*;
use ksync::Mutex;
use pconst::{
    io::{PollEvents, RtcTime, SeekFrom},
    net::*,
};
use rref::{RRef, RRefVec};
use spin::Once;
use task_meta::TaskSchedulingInfo;
use vfscore::{fstype::FileSystemFlags, inode::InodeAttr, superblock::SuperType, utils::*};

use crate::{
    domain_helper::free_domain_resource,
    domain_loader::loader::DomainLoader,
    error::{AlienError, AlienResult},
    sync::{RcuData, SRcuLock},
    task::continuation,
};

pub trait ProxyBuilder {
    type T;
    fn build(id: u64, domain: Self::T, domain_loader: DomainLoader) -> Self;
    fn build_empty(id: u64, domain_loader: DomainLoader) -> Self;
}

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
gen_for_LogDomain!();
gen_for_NetDomain!();
impl_for_FsDomain!(DevFsDomainProxy);

impl_empty_for_FsDomain!(DevFsDomainEmptyImpl);
impl Basic for DevFsDomainEmptyImpl {
    fn is_active(&self) -> bool {
        false
    }
}
impl Basic for DevFsDomainProxy {
    fn is_active(&self) -> bool {
        let idx = self.srcu_lock.read_lock();
        let res = self.domain.get().is_active();
        self.srcu_lock.read_unlock(idx);
        res
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
        new_id: u64,
    ) -> AlienResult<()> {
        let mut loader_guard = self.domain_loader.lock();

        let old_id = self.domain_id();
        self.id.store(new_id, core::sync::atomic::Ordering::SeqCst);
        let old_domain = self.domain.swap(Box::new(new_domain));
        // synchronize the reader which is reading the old domain
        // println!("srcu synchronize");
        self.srcu_lock.synchronize();
        // println!("srcu synchronize end");

        // forget the old domain
        // it will be dropped by the `free_domain_resource`
        let real_domain = Box::into_inner(old_domain);
        forget(real_domain);

        free_domain_resource(old_id);

        let resource = self.resource.get().unwrap();
        self.domain.get().init(resource.as_str()).unwrap();

        *loader_guard = loader;
        Ok(())
    }
}

impl ProxyExt for BlkDomainProxy {
    fn reload(&self) -> AlienResult<()> {
        let mut loader_guard = self.domain_loader.lock();
        let mut loader = loader_guard.clone();
        loader.load().unwrap();
        let id = self.domain_id();
        // todo!(recycle old loader)?
        let new_domain = loader.call(id);

        let new_domain = Box::new(new_domain);
        let old_domain = self.domain.swap(new_domain);
        // synchronize the reader which is reading the old domain
        self.srcu_lock.synchronize();

        // forget the old domain
        let real_domain = Box::into_inner(old_domain);
        forget(real_domain);

        free_domain_resource(id);

        let device_info = self.resource.get().unwrap();
        self.domain.get().init(device_info.clone()).unwrap();

        *loader_guard = loader;
        Ok(())
    }
}

impl SchedulerDomainProxy {
    pub fn replace(
        &self,
        _new_domain: Box<dyn SchedulerDomain>,
        _loader: DomainLoader,
    ) -> AlienResult<()> {
        // let old_domain = self.domain.write();
        // // let mut old = self.old_domain.lock();
        // // *self.domain_loader.lock() = loader;
        // // // swap the old domain with the new one
        // // // and push the old domain to the old domain list( we will fix it)
        // // old.push(core::mem::replace(&mut *old_domain, new_domain));
        // // old_domain.init()
        // println!("Try dump old domain data");
        // let mut data = SchedulerDataContainer::default();
        // old_domain.dump_meta_data(&mut data)?;
        // println!("old domain data: {:?}", data);
        Err(AlienError::EINVAL)
    }
}
impl LogDomainProxy {
    pub fn replace(
        &self,
        new_domain: Box<dyn LogDomain>,
        loader: DomainLoader,
        new_id: u64,
    ) -> AlienResult<()> {
        let mut loader_guard = self.domain_loader.lock();
        let old_id = self.domain_id();
        self.id.store(new_id, core::sync::atomic::Ordering::SeqCst);
        let old_domain = self.domain.swap(Box::new(new_domain));
        // synchronize the reader which is reading the old domain
        self.srcu_lock.synchronize();

        // forget the old domain
        let real_domain = Box::into_inner(old_domain);
        forget(real_domain);

        // free the old domain resource
        free_domain_resource(old_id);

        self.domain.get().init().unwrap();
        *loader_guard = loader;
        Ok(())
    }
}

impl GpuDomainProxy {
    pub fn replace(
        &self,
        new_domain: Box<dyn GpuDomain>,
        loader: DomainLoader,
        new_id: u64,
    ) -> AlienResult<()> {
        let mut loader_guard = self.domain_loader.lock();
        let old_id = self.domain_id();
        self.id.store(new_id, core::sync::atomic::Ordering::SeqCst);
        let old_domain = self.domain.swap(Box::new(new_domain));
        // synchronize the reader which is reading the old domain
        self.srcu_lock.synchronize();

        // forget the old domain
        let real_domain = Box::into_inner(old_domain);
        forget(real_domain);

        // free the old domain resource
        free_domain_resource(old_id);

        let device_info = self.resource.get().unwrap();
        self.domain.get().init(device_info.clone()).unwrap();

        *loader_guard = loader;
        Ok(())
    }
}
