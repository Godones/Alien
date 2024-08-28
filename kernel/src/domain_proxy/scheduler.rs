use alloc::boxed::Box;
use core::{any::Any, fmt::Debug, mem::forget};

use interface::*;
use jtable::*;
use ksync::RwLock;
use paste::paste;
use rref::{RRef, SharedData};
use task_meta::TaskSchedulingInfo;

use crate::{
    domain_helper::{free_domain_resource, FreeShared},
    domain_loader::loader::DomainLoader,
    domain_proxy::{PerCpuCounter, ProxyBuilder},
    error::{AlienError, AlienResult},
    k_static_branch_disable, k_static_branch_enable,
    sync::{synchronize_sched, RcuData, SleepMutex},
    task::yield_now,
};

define_static_key_false!(SCHEDULER_DOMAIN_PROXY_KEY);
#[derive(Debug)]
pub struct SchedulerDomainProxy {
    domain: RcuData<Box<dyn SchedulerDomain>>,
    lock: RwLock<()>,
    domain_loader: SleepMutex<DomainLoader>,
    counter: PerCpuCounter,
}
impl SchedulerDomainProxy {
    pub fn new(domain: Box<dyn SchedulerDomain>, domain_loader: DomainLoader) -> Self {
        Self {
            domain: RcuData::new(Box::new(domain)),
            lock: RwLock::new(()),
            domain_loader: SleepMutex::new(domain_loader),
            counter: PerCpuCounter::new(),
        }
    }
    pub fn all_counter(&self) -> usize {
        self.counter.all()
    }
}
impl ProxyBuilder for SchedulerDomainProxy {
    type T = Box<dyn SchedulerDomain>;
    fn build(domain: Self::T, domain_loader: DomainLoader) -> Self {
        Self::new(domain, domain_loader)
    }
    fn build_empty(domain_loader: DomainLoader) -> Self {
        let domain = Box::new(SchedulerDomainEmptyImpl::new());
        Self::new(domain, domain_loader)
    }
    fn init_by_box(&self, argv: Box<dyn Any + Send + Sync>) -> AlienResult<()> {
        let _ = argv;
        self.init()?;
        Ok(())
    }
}
impl Basic for SchedulerDomainProxy {
    fn domain_id(&self) -> u64 {
        self.domain.get().domain_id()
    }
    fn is_active(&self) -> bool {
        self.domain.get().is_active()
    }
}
impl SchedulerDomain for SchedulerDomainProxy {
    fn init(&self) -> AlienResult<()> {
        self.domain.get().init()
    }
    fn add_task(&self, scheduling_info: RRef<TaskSchedulingInfo>) -> AlienResult<()> {
        if static_branch_likely!(SCHEDULER_DOMAIN_PROXY_KEY) {
            return self.__add_task_with_lock(scheduling_info);
        }
        self.__add_task_no_lock(scheduling_info)
    }
    fn fetch_task(&self, info: RRef<TaskSchedulingInfo>) -> AlienResult<RRef<TaskSchedulingInfo>> {
        if static_branch_likely!(SCHEDULER_DOMAIN_PROXY_KEY) {
            return self.__fetch_task_with_lock(info);
        }
        self.__fetch_task_no_lock(info)
    }
}
impl SchedulerDomainProxy {
    fn __add_task(&self, scheduling_info: RRef<TaskSchedulingInfo>) -> AlienResult<()> {
        let r_domain = self.domain.get();
        let id = r_domain.domain_id();
        let old_id = scheduling_info.move_to(id);

        r_domain.add_task(scheduling_info).map(|r| {
            r.move_to(old_id);
            r
        })
    }
    fn __add_task_no_lock(&self, scheduling_info: RRef<TaskSchedulingInfo>) -> AlienResult<()> {
        self.counter.inc();
        let res = self.__add_task(scheduling_info);
        self.counter.dec();
        res
    }
    #[cold]
    fn __add_task_with_lock(&self, scheduling_info: RRef<TaskSchedulingInfo>) -> AlienResult<()> {
        let r_lock = self.lock.read();
        let res = self.__add_task(scheduling_info);
        drop(r_lock);
        res
    }
    fn __fetch_task(
        &self,
        info: RRef<TaskSchedulingInfo>,
    ) -> AlienResult<RRef<TaskSchedulingInfo>> {
        let r_domain = self.domain.get();
        let id = r_domain.domain_id();
        let old_id = info.move_to(id);

        r_domain.fetch_task(info).map(|r| {
            r.move_to(old_id);
            r
        })
    }
    fn __fetch_task_no_lock(
        &self,
        info: RRef<TaskSchedulingInfo>,
    ) -> AlienResult<RRef<TaskSchedulingInfo>> {
        self.counter.inc();
        let res = self.__fetch_task(info);
        self.counter.dec();
        res
    }
    #[cold]
    fn __fetch_task_with_lock(
        &self,
        info: RRef<TaskSchedulingInfo>,
    ) -> AlienResult<RRef<TaskSchedulingInfo>> {
        let r_lock = self.lock.read();
        let res = self.__fetch_task(info);
        drop(r_lock);
        res
    }
}
#[derive(Debug)]
struct SchedulerDomainEmptyImpl;
impl SchedulerDomainEmptyImpl {
    pub fn new() -> Self {
        Self
    }
}
impl Basic for SchedulerDomainEmptyImpl {
    fn domain_id(&self) -> u64 {
        u64::MAX
    }
    fn is_active(&self) -> bool {
        false
    }
}
impl SchedulerDomain for SchedulerDomainEmptyImpl {
    fn init(&self) -> AlienResult<()> {
        Ok(())
    }
    #[doc = " add one task to scheduler"]
    fn add_task(&self, _scheduling_info: RRef<TaskSchedulingInfo>) -> AlienResult<()> {
        Err(AlienError::ENOSYS)
    }
    #[doc = " The next task to run"]
    fn fetch_task(&self, _info: RRef<TaskSchedulingInfo>) -> AlienResult<RRef<TaskSchedulingInfo>> {
        Err(AlienError::ENOSYS)
    }
}

impl SchedulerDomainProxy {
    pub fn replace(
        &self,
        new_domain: Box<dyn SchedulerDomain>,
        loader: DomainLoader,
    ) -> AlienResult<()> {
        // stage1: get the sleep lock and change to updating state
        let mut loader_guard = self.domain_loader.lock();

        k_static_branch_enable!(SCHEDULER_DOMAIN_PROXY_KEY);

        // why we need to synchronize_sched here?
        synchronize_sched();

        // stage2: get the write lock and wait for all readers to finish
        let w_lock = self.lock.write();
        // wait if there are readers which are reading the old domain but no read lock
        // todo!( "wait for all reader to finish");
        while self.all_counter() > 0 {
            println!("Wait for all reader to finish");
            yield_now();
        }
        let old_id = self.domain_id();

        // stage3: init the new domain before swap
        let new_domain_id = new_domain.domain_id();
        new_domain.init().unwrap();

        // stage4: swap the domain and change to normal state
        let old_domain = self.domain.swap(Box::new(new_domain));

        // change to normal state
        k_static_branch_disable!(SCHEDULER_DOMAIN_PROXY_KEY);

        // stage5: recycle all resources
        let real_domain = Box::into_inner(old_domain);
        // forget the old domain, it will be dropped by the `free_domain_resource`
        forget(real_domain);

        // todo!(how to recycle the old domain)
        // We should not free the shared data here, because the shared data will be used
        // in new domain.
        free_domain_resource(old_id, FreeShared::NotFree(new_domain_id));

        // stage6: release all locks
        *loader_guard = loader;
        drop(w_lock);
        drop(loader_guard);
        Ok(())
    }
}
