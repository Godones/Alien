mod scheduler;

use alloc::{boxed::Box, string::String, vec::Vec};
use core::{any::Any, cell::UnsafeCell, fmt::Debug, mem::forget, net::SocketAddrV4, ops::Range};

use arch::hart_id;
use config::CPU_NUM;
use interface::*;
use jtable::*;
use ksync::{Mutex, RwLock};
use paste::paste;
use pconst::{
    epoll::EpollEvent,
    io::{PollEvents, RtcTime, SeekFrom},
    net::*,
};
pub use scheduler::SchedulerDomainProxy;
use shared_heap::{DBox, DVec, SharedData};
use spin::Once;
use vfscore::{fstype::FileSystemFlags, inode::InodeAttr, superblock::SuperType, utils::*};

use crate::{
    domain_helper::{free_domain_resource, FreeShared},
    domain_loader::loader::DomainLoader,
    error::{AlienError, AlienResult},
    sync::{synchronize_sched, RcuData, SRcuLock, SleepMutex},
    task::yield_now,
    timer::TimeTick,
    *,
};
pub trait ProxyBuilder {
    type T;
    fn build(domain: Self::T, domain_loader: DomainLoader) -> Self;
    fn build_empty(domain_loader: DomainLoader) -> Self;
    fn init_by_box(&self, argv: Box<dyn Any + Send + Sync>) -> AlienResult<()>;
}

#[derive(Debug)]
pub struct PerCpuCounter {
    counter: [UnsafeCell<isize>; CPU_NUM],
}

unsafe impl Sync for PerCpuCounter {}

const PER_CPU_VALUE: UnsafeCell<isize> = UnsafeCell::new(0);
impl PerCpuCounter {
    pub const fn new() -> Self {
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
        // assert!(v as isize - 1 >= 0);
        write_once!(self.counter[hart_id()].get(), v - 1);
    }
    pub fn all(&self) -> usize {
        let mut sum = 0;
        for i in 0..CPU_NUM {
            sum += read_once!(self.counter[i].get());
        }
        sum as usize
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
// show how to use the macro
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

impl BlkDomainProxy {
    pub fn reload(
        &self,
        new_domain: Box<dyn BlkDeviceDomain>,
        loader: DomainLoader,
    ) -> AlienResult<()> {
        // stage1: get the sleep lock and change to updating state
        let mut loader_guard = self.domain_loader.lock();

        // stage2: get the write lock and wait for all readers to finish
        let w_lock = self.lock.write();

        static_branch_enable!(BLKDOMAINPROXY_KEY);

        // why we need to synchronize_sched here?
        synchronize_sched();

        // wait if there are readers which are reading the old domain but no read lock
        while self.all_counter() > 0 {
            // println!("Wait for all reader to finish");
            // yield_now();
        }

        let old_id = self.domain_id();

        // stage3: init the new domain before swap
        let device_info = self.resource.get().unwrap();
        let info = device_info.as_ref().downcast_ref::<Range<usize>>().unwrap();
        new_domain.init(info).unwrap();

        // stage4: swap the domain and change to normal state
        let old_domain = self.domain.swap(Box::new(new_domain));
        // change to normal state
        static_branch_disable!(BLKDOMAINPROXY_KEY);

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

// impl ShadowBlockDomainProxy {
//     pub fn replace(
//         &self,
//         new_domain: Box<dyn ShadowBlockDomain>,
//         loader: DomainLoader,
//     ) -> AlienResult<()> {
//         let mut loader_guard = self.domain_loader.lock();
//         let old_id = self.domain_id();
//
//         // init the new domain before swap
//         let resource = self.resource.get().unwrap();
//         let info = resource.as_ref().downcast_ref::<String>().unwrap();
//         new_domain.init(info).unwrap();
//
//         let old_domain = self.domain.swap(Box::new(new_domain));
//         // synchronize the reader which is reading the old domain
//         // println!("srcu synchronize");
//         self.srcu_lock.synchronize();
//         // println!("srcu synchronize end");
//
//         // forget the old domain
//         // it will be dropped by the `free_domain_resource`
//         let real_domain = Box::into_inner(old_domain);
//         forget(real_domain);
//
//         free_domain_resource(old_id, FreeShared::Free);
//         *loader_guard = loader;
//         Ok(())
//     }
// }
