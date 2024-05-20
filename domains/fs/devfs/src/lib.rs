#![no_std]
#![forbid(unsafe_code)]

mod block;
mod empty_device;
mod gpu;
mod r#impl;
mod input;
mod rtc;
mod uart;

extern crate alloc;

use alloc::{
    boxed::Box,
    collections::BTreeMap,
    string::{String, ToString},
    sync::Arc,
};
use core::fmt::Debug;

use basic::sync::Mutex;
use devfs::{DevFs, DevKernelProvider};
use generic::GenericFsDomain;
use interface::{DevFsDomain, DomainType, TaskDomain};
use log::info;
use spin::{Lazy, Once};
use vfscore::{inode::VfsInode, utils::VfsTimeSpec};

use crate::{
    block::BLKDevice, empty_device::EmptyDevice, gpu::GPUDevice, input::INPUTDevice,
    r#impl::DevFsDomainImpl, rtc::RTCDevice, uart::UARTDevice,
};

#[derive(Clone, Debug)]
pub struct ProviderImpl;

impl DevKernelProvider for ProviderImpl {
    fn current_time(&self) -> VfsTimeSpec {
        VfsTimeSpec::new(0, 0)
    }

    fn rdev2device(&self, rdev: u64) -> Option<Arc<dyn VfsInode>> {
        let mut dev_shim = DEV_INODE_MAP.lock();
        match dev_shim.get(&rdev) {
            Some(inode) => Some(inode.clone()),
            None => {
                let dev_map = DEV_MAP.lock();
                let device_domain_name = dev_map.get(&rdev)?;
                let device_domain = basic::get_domain(device_domain_name.as_str())?;
                match device_domain {
                    DomainType::CacheBlkDeviceDomain(blk) => {
                        let dev = Arc::new(BLKDevice::new(rdev.into(), blk));
                        dev_shim.insert(rdev, dev.clone());
                        Some(dev)
                    }
                    DomainType::BufUartDomain(uart) => {
                        let task_domain = TASK_DOMAIN.get().unwrap().clone();
                        let dev = Arc::new(UARTDevice::new(rdev.into(), uart, task_domain));
                        dev_shim.insert(rdev, dev.clone());
                        Some(dev)
                    }
                    DomainType::RtcDomain(rtc) => {
                        let task_domain = TASK_DOMAIN.get().unwrap().clone();
                        let dev = Arc::new(RTCDevice::new(rdev.into(), rtc, task_domain));
                        dev_shim.insert(rdev, dev.clone());
                        Some(dev)
                    }
                    DomainType::GpuDomain(gpu) => {
                        let dev = Arc::new(GPUDevice::new(rdev.into(), gpu));
                        dev_shim.insert(rdev, dev.clone());
                        Some(dev)
                    }
                    DomainType::BufInputDomain(input) => {
                        let dev = Arc::new(INPUTDevice::new(rdev.into(), input));
                        dev_shim.insert(rdev, dev.clone());
                        Some(dev)
                    }
                    DomainType::EmptyDeviceDomain(empty) => {
                        let dev = Arc::new(EmptyDevice::new(rdev.into(), empty));
                        dev_shim.insert(rdev, dev.clone());
                        Some(dev)
                    }
                    ty => {
                        info!("{} not found, found {:?}", device_domain_name, ty);
                        None
                    }
                }
            }
        }
    }
}
static DEV_INODE_MAP: Lazy<Mutex<BTreeMap<u64, Arc<dyn VfsInode>>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));
static DEV_MAP: Lazy<Mutex<BTreeMap<u64, String>>> = Lazy::new(|| Mutex::new(BTreeMap::new()));

static TASK_DOMAIN: Once<Arc<dyn TaskDomain>> = Once::new();

pub fn main() -> Box<dyn DevFsDomain> {
    let devfs = Arc::new(DevFs::<_, Mutex<()>>::new(ProviderImpl));
    let devfs = GenericFsDomain::new(devfs, "devfs".to_string(), None);
    Box::new(DevFsDomainImpl::new(devfs))
}
