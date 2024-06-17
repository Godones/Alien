use alloc::{boxed::Box, vec::Vec};
use core::{any::Any, mem::forget, ops::Deref, sync::atomic::AtomicBool};

use arch::hart_id;
use config::CPU_NUM;
use interface::{Basic, SchedulerDomain};
use ksync::{RwLock, SafeRefCell};
use rref::{RRef, SharedData};
use task_meta::TaskSchedulingInfo;

use crate::{
    domain_helper::free_domain_resource,
    domain_loader::loader::DomainLoader,
    domain_proxy::ProxyBuilder,
    error::{AlienError, AlienResult},
    sync::{RcuData, SleepMutex},
};

#[derive(Debug)]
pub struct SchedulerDomainProxy {
    in_updating: AtomicBool,
    domain: RcuData<Box<dyn SchedulerDomain>>,
    lock: RwLock<()>,
    domain_loader: SleepMutex<DomainLoader>,
    per_cpu_counter: [SafeRefCell<usize>; CPU_NUM],
}
impl SchedulerDomainProxy {
    pub fn new(domain: Box<dyn SchedulerDomain>, domain_loader: DomainLoader) -> Self {
        const PER_CPU_VALUE: SafeRefCell<usize> = SafeRefCell::new(0);
        Self {
            in_updating: AtomicBool::new(false),
            domain: RcuData::new(Box::new(domain)),
            lock: RwLock::new(()),
            domain_loader: SleepMutex::new(domain_loader),
            per_cpu_counter: [PER_CPU_VALUE; CPU_NUM],
        }
    }

    pub fn is_updating(&self) -> bool {
        self.in_updating.load(core::sync::atomic::Ordering::Relaxed)
    }

    pub fn all_counter(&self) -> usize {
        self.per_cpu_counter.iter().map(|x| *x.borrow()).sum()
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
        let id = || self.domain.get().domain_id();
        if !self.is_updating() {
            self.per_cpu_counter[hart_id()]
                .deref()
                .replace_with(|x| *x + 1);
            let res = id();
            self.per_cpu_counter[hart_id()]
                .deref()
                .replace_with(|x| *x - 1);
            return res;
        }
        let r_lock = self.lock.read();
        let res = id();
        drop(r_lock);
        res
    }
    fn is_active(&self) -> bool {
        let is_active = || self.domain.get().is_active();
        if !self.is_updating() {
            self.per_cpu_counter[hart_id()]
                .deref()
                .replace_with(|x| *x + 1);
            let res = is_active();
            self.per_cpu_counter[hart_id()]
                .deref()
                .replace_with(|x| *x - 1);
            return res;
        }
        let r_lock = self.lock.read();
        let res = is_active();
        drop(r_lock);
        res
    }
}
impl SchedulerDomain for SchedulerDomainProxy {
    fn init(&self) -> AlienResult<()> {
        self.domain.get().init()
    }
    fn add_task(&self, scheduling_info: RRef<TaskSchedulingInfo>) -> AlienResult<()> {
        let add_task = || {
            let domain = self.domain.get();
            if !domain.is_active() {
                return Err(AlienError::DOMAINCRASH);
            }
            let id = domain.domain_id();
            let old_id = scheduling_info.move_to(id);
            let res = domain.add_task(scheduling_info).map(|r| {
                r.move_to(old_id);
                r
            });
            return res;
        };
        if !self.is_updating() {
            self.per_cpu_counter[hart_id()]
                .deref()
                .replace_with(|x| *x + 1);
            let res = add_task();
            self.per_cpu_counter[hart_id()]
                .deref()
                .replace_with(|x| *x - 1);
            return res;
        }
        let r_lock = self.lock.read();
        let res = add_task();
        drop(r_lock);
        res
    }
    fn fetch_task(&self, info: RRef<TaskSchedulingInfo>) -> AlienResult<RRef<TaskSchedulingInfo>> {
        let fetch_task = || {
            let domain = self.domain.get();
            if !domain.is_active() {
                return Err(AlienError::DOMAINCRASH);
            }
            let id = domain.domain_id();
            let old_id = info.move_to(id);
            let res = domain.fetch_task(info).map(|r| {
                r.move_to(old_id);
                r
            });
            return res;
        };

        if !self.is_updating() {
            self.per_cpu_counter[hart_id()]
                .deref()
                .replace_with(|x| *x + 1);
            let res = fetch_task();
            self.per_cpu_counter[hart_id()]
                .deref()
                .replace_with(|x| *x - 1);
            return res;
        }
        let r_lock = self.lock.read();
        let res = fetch_task();
        drop(r_lock);
        res
    }

    fn dump_meta_data(&self) -> AlienResult<Vec<RRef<TaskSchedulingInfo>>> {
        unimplemented!()
    }

    fn rebuild_from_meta_data(
        &self,
        _meta_data: &mut Vec<RRef<TaskSchedulingInfo>>,
    ) -> AlienResult<()> {
        unimplemented!()
    }
}

impl SchedulerDomainProxy {
    pub fn replace(
        &self,
        new_domain: Box<dyn SchedulerDomain>,
        loader: DomainLoader,
    ) -> AlienResult<()> {
        let mut loader_guard = self.domain_loader.lock();
        // change to updating state
        self.in_updating
            .store(true, core::sync::atomic::Ordering::Relaxed);
        // try get write lock
        let w_lock = self.lock.write();
        // wait if there are readers which are reading the old domain but no read lock
        while self.all_counter() > 0 {
            println!("Wait for all reader to finish");
            core::hint::spin_loop();
        }
        // now there is no reader reading the old domain
        println!("Try dump old domain data");
        let domain = self.domain.get();
        let old_id = domain.domain_id();

        let mut task_list = domain.dump_meta_data()?;

        println!("old domain data: {:?}", task_list);
        let old_domain = self.domain.swap(Box::new(new_domain));

        println!("Try rebuild from meta data");
        self.domain.get().rebuild_from_meta_data(&mut task_list)?;

        let real_domain = Box::into_inner(old_domain);
        // forget the old domain
        // it will be dropped by the `free_domain_resource`
        forget(real_domain);
        forget(task_list); // it also will be dropped by the `free_domain_resource`

        free_domain_resource(old_id);

        // recover to normal state
        self.in_updating
            .store(false, core::sync::atomic::Ordering::Relaxed);

        // replace the domain
        *loader_guard = loader;
        // release the write lock
        drop(w_lock);
        // release the loader guard
        drop(loader_guard);
        // panic!("Try to replace scheduler domain ok");
        Ok(())
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
    fn add_task(&self, _scheduling_info: RRef<TaskSchedulingInfo>) -> AlienResult<()> {
        Err(AlienError::ENOSYS)
    }
    fn fetch_task(&self, _info: RRef<TaskSchedulingInfo>) -> AlienResult<RRef<TaskSchedulingInfo>> {
        Err(AlienError::ENOSYS)
    }

    fn dump_meta_data(&self) -> AlienResult<Vec<RRef<TaskSchedulingInfo>>> {
        Err(AlienError::ENOSYS)
    }

    fn rebuild_from_meta_data(
        &self,
        _meta_data: &mut Vec<RRef<TaskSchedulingInfo>>,
    ) -> AlienResult<()> {
        Err(AlienError::ENOSYS)
    }
}
