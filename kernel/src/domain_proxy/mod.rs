mod scheduler;

use alloc::{boxed::Box, string::String, vec::Vec};
use core::{
    any::Any, cell::UnsafeCell, fmt::Debug, mem::forget, net::SocketAddrV4, ops::Range,
    sync::atomic::AtomicBool,
};

use arch::hart_id;
use config::CPU_NUM;
use downcast_rs::{impl_downcast, DowncastSync};
use interface::*;
use ksync::{Mutex, RwLock};
use pconst::{
    io::{PollEvents, RtcTime, SeekFrom},
    net::*,
};
use rref::{RRef, RRefVec, SharedData};
pub use scheduler::SchedulerDomainProxy;
use spin::Once;
use vfscore::{fstype::FileSystemFlags, inode::InodeAttr, superblock::SuperType, utils::*};

use crate::{
    domain_helper::{alloc_domain_id, free_domain_resource, FreeShared},
    domain_loader::loader::DomainLoader,
    error::{AlienError, AlienResult},
    read_once,
    sync::{synchronize_sched, RcuData, SRcuLock, SleepMutex},
    task::{continuation, yield_now},
    write_once,
};

pub trait ProxyBuilder {
    type T;
    fn build(domain: Self::T, domain_loader: DomainLoader) -> Self;
    fn build_empty(domain_loader: DomainLoader) -> Self;
    fn init_by_box(&self, argv: Box<dyn Any + Send + Sync>) -> AlienResult<()>;
}

#[derive(Debug)]
pub struct PerCpuCounter {
    counter: [UnsafeCell<usize>; CPU_NUM],
}
unsafe impl Sync for PerCpuCounter {}

impl PerCpuCounter {
    pub const fn new() -> Self {
        const PER_CPU_VALUE: UnsafeCell<usize> = UnsafeCell::new(0);
        Self {
            counter: [PER_CPU_VALUE; CPU_NUM],
        }
    }
    pub fn inc(&self) {
        let v = read_once!(self.counter[hart_id()].get());
        write_once!(self.counter[hart_id()].get(), v + 1);
    }
    pub fn dec(&self) {
        let v = read_once!(self.counter[hart_id()].get());
        write_once!(self.counter[hart_id()].get(), v - 1);
    }
    pub fn all(&self) -> usize {
        let mut sum = 0;
        for i in 0..CPU_NUM {
            sum += read_once!(self.counter[i].get());
        }
        sum
    }
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
// gen_for_SchedulerDomain!();
gen_for_ShadowBlockDomain!();
gen_for_BlkDeviceDomain!();

gen_for_DevFsDomain!();
gen_for_LogDomain!();
gen_for_NetDomain!();
impl_for_FsDomain!(DevFsDomainProxy);

impl_empty_for_FsDomain!(DevFsDomainEmptyImpl);
impl Basic for DevFsDomainEmptyImpl {
    fn domain_id(&self) -> u64 {
        u64::MAX
    }
    fn is_active(&self) -> bool {
        false
    }
}
impl Basic for DevFsDomainProxy {
    fn domain_id(&self) -> u64 {
        self.domain.get().domain_id()
    }
    fn is_active(&self) -> bool {
        self.domain.get().is_active()
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
        let mut loader_guard = self.domain_loader.lock();
        let old_id = self.domain_id();

        // init the new domain before swap
        let resource = self.resource.get().unwrap();
        let info = resource.as_ref().downcast_ref::<String>().unwrap();
        new_domain.init(info).unwrap();

        let old_domain = self.domain.swap(Box::new(new_domain));
        // synchronize the reader which is reading the old domain
        // println!("srcu synchronize");
        self.srcu_lock.synchronize();
        // println!("srcu synchronize end");

        // forget the old domain
        // it will be dropped by the `free_domain_resource`
        let real_domain = Box::into_inner(old_domain);
        forget(real_domain);

        free_domain_resource(old_id, FreeShared::Free);
        *loader_guard = loader;
        Ok(())
    }
}

impl LogDomainProxy {
    pub fn replace(&self, new_domain: Box<dyn LogDomain>, loader: DomainLoader) -> AlienResult<()> {
        let mut loader_guard = self.domain_loader.lock();
        let old_id = self.domain_id();

        // init the new domain before swap
        new_domain.init().unwrap();

        let old_domain = self.domain.swap(Box::new(new_domain));
        // synchronize the reader which is reading the old domain
        self.srcu_lock.synchronize();

        // forget the old domain
        let real_domain = Box::into_inner(old_domain);
        forget(real_domain);

        // free the old domain resource
        free_domain_resource(old_id, FreeShared::Free);

        *loader_guard = loader;
        Ok(())
    }
}
impl ProxyExt for BlkDomainProxy {
    fn reload(&self) -> AlienResult<()> {
        // stage1: get the sleep lock and change to updating state
        let mut loader_guard = self.domain_loader.lock();
        self.in_updating
            .store(true, core::sync::atomic::Ordering::Relaxed);

        // why we need to synchronize_sched here?
        synchronize_sched();

        // stage2: get the write lock and wait for all readers to finish
        let w_lock = self.lock.write();
        // wait if there are readers which are reading the old domain but no read lock
        while self.all_counter() > 0 {
            println!("Wait for all reader to finish");
            yield_now();
        }

        let old_id = self.domain_id();

        // stage3: init the new domain before swap
        // 1. create the new domain
        let mut loader = loader_guard.clone();
        loader.load().unwrap();
        let new_id = alloc_domain_id();
        let new_domain = loader.call::<dyn BlkDeviceDomain>(new_id, Some(old_id));
        // 2. init the new domain
        let device_info = self.resource.get().unwrap();
        let info = device_info.as_ref().downcast_ref::<Range<usize>>().unwrap();
        new_domain.init(info).unwrap();

        // stage4: swap the domain and change to normal state
        let old_domain = self.domain.swap(Box::new(new_domain));
        // change to normal state
        self.in_updating
            .store(false, core::sync::atomic::Ordering::Relaxed);

        // stage5: recycle all resources
        let real_domain = Box::into_inner(old_domain);
        forget(real_domain);
        free_domain_resource(old_id, FreeShared::Free);

        // stage6: release all locks
        *loader_guard = loader;
        drop(w_lock);
        drop(loader_guard);
        Ok(())
    }
}

impl GpuDomainProxy {
    pub fn replace(&self, new_domain: Box<dyn GpuDomain>, loader: DomainLoader) -> AlienResult<()> {
        // stage1: get the sleep lock and change to updating state
        let mut loader_guard = self.domain_loader.lock();
        self.in_updating
            .store(true, core::sync::atomic::Ordering::Relaxed);

        // why we need to synchronize_sched here?
        synchronize_sched();

        // stage2: get the write lock and wait for all readers to finish
        let w_lock = self.lock.write();
        // wait if there are readers which are reading the old domain but no read lock
        while self.all_counter() > 0 {
            println!("Wait for all reader to finish");
            yield_now();
        }
        let old_id = self.domain_id();

        // stage3: init the new domain before swap
        let new_domain_id = new_domain.domain_id();
        let device_info = self.resource.get().unwrap();
        let info = device_info.as_ref().downcast_ref::<Range<usize>>().unwrap();
        new_domain.init(info).unwrap();

        // stage4: swap the domain and change to normal state
        let old_domain = self.domain.swap(Box::new(new_domain));
        // change to normal state
        self.in_updating
            .store(false, core::sync::atomic::Ordering::Relaxed);

        // stage5: recycle all resources
        // forget the old domain
        let real_domain = Box::into_inner(old_domain);
        forget(real_domain);
        free_domain_resource(old_id, FreeShared::NotFree(new_domain_id));

        // stage6: release all locks
        *loader_guard = loader;
        drop(w_lock);
        drop(loader_guard);
        Ok(())
    }
}

impl InputDomainProxy {
    pub fn replace(
        &self,
        new_domain: Box<dyn InputDomain>,
        loader: DomainLoader,
    ) -> AlienResult<()> {
        // stage1: get the sleep lock and change to updating state
        let mut loader_guard = self.domain_loader.lock();
        // change to updating state
        self.in_updating
            .store(true, core::sync::atomic::Ordering::Relaxed);

        // why we need to synchronize_sched here?
        synchronize_sched();

        // stage2: get the write lock and wait for all readers to finish
        let w_lock = self.lock.write();
        // wait if there are readers which are reading the old domain but no read lock
        while self.all_counter() > 0 {
            println!("Wait for all reader to finish");
            yield_now();
        }
        let old_id = self.domain_id();

        // stage3: init the new domain before swap
        let new_domain_id = new_domain.domain_id();
        let device_info = self.resource.get().unwrap();
        let info = device_info.as_ref().downcast_ref::<Range<usize>>().unwrap();
        new_domain.init(info).unwrap();

        // stage4: swap the domain and change to normal state
        let old_domain = self.domain.swap(Box::new(new_domain));

        // change to normal state
        self.in_updating
            .store(false, core::sync::atomic::Ordering::Relaxed);

        // stage5: recycle all resources
        let real_domain = Box::into_inner(old_domain);
        forget(real_domain);
        free_domain_resource(old_id, FreeShared::NotFree(new_domain_id));

        // stage6: release all locks
        *loader_guard = loader;
        drop(w_lock);
        drop(loader_guard);
        Ok(())
    }
}

impl NetDeviceDomainProxy {
    pub fn replace(
        &self,
        new_domain: Box<dyn NetDeviceDomain>,
        loader: DomainLoader,
    ) -> AlienResult<()> {
        // stage1: get the sleep lock and change to updating state
        let mut loader_guard = self.domain_loader.lock();
        // change to updating state
        self.in_updating
            .store(true, core::sync::atomic::Ordering::Relaxed);

        // why we need to synchronize_sched here?
        synchronize_sched();

        // stage2: get the write lock and wait for all readers to finish
        let w_lock = self.lock.write();
        // wait if there are readers which are reading the old domain but no read lock
        while self.all_counter() > 0 {
            println!("Wait for all reader to finish");
            yield_now();
        }
        let old_id = self.domain_id();

        // stage3: init the new domain before swap
        let new_domain_id = new_domain.domain_id();
        let device_info = self.resource.get().unwrap();
        let info = device_info.as_ref().downcast_ref::<Range<usize>>().unwrap();
        new_domain.init(info).unwrap();

        // stage4: swap the domain and change to normal state
        let old_domain = self.domain.swap(Box::new(new_domain));

        // change to normal state
        self.in_updating
            .store(false, core::sync::atomic::Ordering::Relaxed);

        // stage5: recycle all resources
        let real_domain = Box::into_inner(old_domain);
        forget(real_domain);
        free_domain_resource(old_id, FreeShared::NotFree(new_domain_id));

        // stage6: release all locks
        *loader_guard = loader;
        drop(w_lock);
        drop(loader_guard);
        Ok(())
    }
}

impl VfsDomainProxy {
    pub fn replace(&self, new_domain: Box<dyn VfsDomain>, loader: DomainLoader) -> AlienResult<()> {
        // stage1: get the sleep lock and change to updating state
        let mut loader_guard = self.domain_loader.lock();
        // change to updating state
        self.in_updating
            .store(true, core::sync::atomic::Ordering::Relaxed);

        // why we need to synchronize_sched here?
        synchronize_sched();

        // stage2: get the write lock and wait for all readers to finish
        let w_lock = self.lock.write();
        // wait if there are readers which are reading the old domain but no read lock
        while self.all_counter() > 0 {
            println!("Wait for all reader to finish");
            yield_now();
        }
        let old_id = self.domain_id();

        // stage3: init the new domain before swap
        let new_domain_id = new_domain.domain_id();
        let device_info = self.resource.get().unwrap();
        let info = device_info.as_ref().downcast_ref::<Vec<u8>>().unwrap();
        new_domain.init(info).unwrap();

        // stage4: swap the domain and change to normal state
        let old_domain = self.domain.swap(Box::new(new_domain));

        // change to normal state
        self.in_updating
            .store(false, core::sync::atomic::Ordering::Relaxed);

        // stage5: recycle all resources
        let real_domain = Box::into_inner(old_domain);
        forget(real_domain);
        free_domain_resource(old_id, FreeShared::NotFree(new_domain_id));

        // stage6: release all locks
        *loader_guard = loader;
        drop(w_lock);
        drop(loader_guard);
        Ok(())
    }
}
