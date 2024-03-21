#![no_std]

mod block;
mod buf_uart;
mod cache_block;
mod devices;
mod gpu;
mod input_device;
mod plic;
mod rtc;
mod shadow_block;
mod syscall;
mod task;
mod uart;
#[allow(unused)]
mod vfs;

extern crate alloc;

use alloc::sync::Arc;
use constants::{AlienError, AlienResult};
use core::any::Any;
use core::fmt::Debug;

pub trait Basic: Send + Sync + Debug + Any {
    // may be deleted
    // fn drop_self(self: Arc<Self>) {
    //     drop(self);
    // }

    #[cfg(feature = "domain")]
    fn is_active(&self) -> bool {
        __impl::is_active()
    }
    #[cfg(not(feature = "domain"))]
    fn is_active(&self) -> bool;
}

pub trait DeviceBase: Basic {
    fn handle_irq(&self) -> AlienResult<()>;
}

#[cfg(feature = "task")]
pub use task::*;

#[cfg(feature = "blk")]
pub use block::BlkDeviceDomain;

#[cfg(feature = "cache_blk")]
pub use cache_block::CacheBlkDeviceDomain;

#[cfg(feature = "fs")]
pub trait FsDomain: Basic {}

#[cfg(feature = "uart")]
pub use uart::UartDomain;

#[cfg(feature = "gpu")]
pub use gpu::GpuDomain;

#[cfg(feature = "input")]
pub use input_device::InputDomain;

#[cfg(feature = "vfs")]
pub use vfs::*;

#[cfg(feature = "rtc")]
pub use rtc::{RtcDomain, RtcTime};

#[cfg(feature = "plic")]
pub use plic::PLICDomain;

#[cfg(feature = "devices")]
pub use devices::{DeviceInfo, DevicesDomain};

#[cfg(feature = "syscall")]
pub use syscall::SysCallDomain;

#[cfg(feature = "shadow_blk")]
pub use shadow_block::ShadowBlockDomain;

#[cfg(feature = "buf_uart")]
pub use buf_uart::BufUartDomain;

#[derive(Clone, Debug)]
pub enum DomainType {
    #[cfg(feature = "fs")]
    FsDomain(Arc<dyn FsDomain>),
    #[cfg(feature = "blk")]
    BlkDeviceDomain(Arc<dyn BlkDeviceDomain>),
    #[cfg(feature = "cache_blk")]
    CacheBlkDeviceDomain(Arc<dyn CacheBlkDeviceDomain>),
    #[cfg(feature = "rtc")]
    RtcDomain(Arc<dyn RtcDomain>),
    #[cfg(feature = "gpu")]
    GpuDomain(Arc<dyn GpuDomain>),
    #[cfg(feature = "input")]
    InputDomain(Arc<dyn InputDomain>),
    #[cfg(feature = "vfs")]
    VfsDomain(Arc<dyn VfsDomain>),
    #[cfg(feature = "uart")]
    UartDomain(Arc<dyn UartDomain>),
    #[cfg(feature = "plic")]
    PLICDomain(Arc<dyn PLICDomain>),
    #[cfg(feature = "devices")]
    DevicesDomain(Arc<dyn DevicesDomain>),
    #[cfg(feature = "task")]
    TaskDomain(Arc<dyn TaskDomain>),
    #[cfg(feature = "syscall")]
    SysCallDomain(Arc<dyn SysCallDomain>),
    #[cfg(feature = "shadow_blk")]
    ShadowBlockDomain(Arc<dyn ShadowBlockDomain>),
    #[cfg(feature = "buf_uart")]
    BufUartDomain(Arc<dyn BufUartDomain>),
}

impl TryInto<Arc<dyn DeviceBase>> for DomainType {
    type Error = AlienError;

    fn try_into(self) -> Result<Arc<dyn DeviceBase>, Self::Error> {
        match self {
            #[cfg(feature = "blk")]
            DomainType::BlkDeviceDomain(domain) => Ok(domain),
            #[cfg(feature = "cache_blk")]
            DomainType::CacheBlkDeviceDomain(domain) => Ok(domain),
            #[cfg(feature = "rtc")]
            DomainType::RtcDomain(domain) => Ok(domain),
            #[cfg(feature = "gpu")]
            DomainType::GpuDomain(domain) => Ok(domain),
            #[cfg(feature = "input")]
            DomainType::InputDomain(domain) => Ok(domain),
            #[cfg(feature = "uart")]
            DomainType::UartDomain(domain) => Ok(domain),
            #[cfg(feature = "shadow_blk")]
            DomainType::ShadowBlockDomain(domain) => Ok(domain),
            #[cfg(feature = "buf_uart")]
            DomainType::BufUartDomain(domain) => Ok(domain),
            _ => Err(AlienError::EINVAL),
        }
    }
}

#[cfg(feature = "domain")]
mod __impl {
    use core::hint::spin_loop;
    use core::sync::atomic::AtomicBool;

    static ACTIVE: AtomicBool = AtomicBool::new(false);

    /// Activate the domain
    ///
    /// It should be called in the `main` function of the domain.
    pub fn activate_domain() {
        ACTIVE.store(true, core::sync::atomic::Ordering::SeqCst);
    }

    pub(super) fn is_active() -> bool {
        ACTIVE.load(core::sync::atomic::Ordering::SeqCst)
    }

    /// Deactivate the domain
    ///
    /// It should be called in the `panic` function of the domain and it should block the thread which
    /// calls this function when the `ACTIVE` flag is false.
    pub fn deactivate_domain() {
        while !ACTIVE.swap(false, core::sync::atomic::Ordering::SeqCst) {
            spin_loop();
        }
    }
}

#[cfg(feature = "domain")]
pub use __impl::*;
