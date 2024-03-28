#![no_std]

mod block;
mod buf_input;
mod buf_uart;
mod cache_block;
mod devices;
mod gpu;
mod input_device;
mod net;
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
use core::{any::Any, fmt::Debug};

use constants::{AlienError, AlienResult};

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

pub use block::BlkDeviceDomain;
pub use cache_block::CacheBlkDeviceDomain;
pub use task::*;

pub trait FsDomain: Basic {}
pub use buf_input::BufInputDomain;
pub use buf_uart::BufUartDomain;
pub use devices::{DeviceInfo, DevicesDomain};
pub use gpu::GpuDomain;
pub use input_device::InputDomain;
pub use net::*;
pub use plic::PLICDomain;
pub use rtc::RtcDomain;
pub use shadow_block::ShadowBlockDomain;
pub use syscall::SysCallDomain;
pub use uart::UartDomain;
pub use vfs::*;

#[derive(Clone, Debug)]
pub enum DomainType {
    FsDomain(Arc<dyn FsDomain>),
    BlkDeviceDomain(Arc<dyn BlkDeviceDomain>),
    CacheBlkDeviceDomain(Arc<dyn CacheBlkDeviceDomain>),
    RtcDomain(Arc<dyn RtcDomain>),
    GpuDomain(Arc<dyn GpuDomain>),
    InputDomain(Arc<dyn InputDomain>),
    VfsDomain(Arc<dyn VfsDomain>),
    UartDomain(Arc<dyn UartDomain>),
    PLICDomain(Arc<dyn PLICDomain>),
    DevicesDomain(Arc<dyn DevicesDomain>),
    TaskDomain(Arc<dyn TaskDomain>),
    SysCallDomain(Arc<dyn SysCallDomain>),
    ShadowBlockDomain(Arc<dyn ShadowBlockDomain>),
    BufUartDomain(Arc<dyn BufUartDomain>),
    NetDomain(Arc<dyn NetDomain>),
    BufInputDomain(Arc<dyn BufInputDomain>),
}

impl TryInto<Arc<dyn DeviceBase>> for DomainType {
    type Error = AlienError;

    fn try_into(self) -> Result<Arc<dyn DeviceBase>, Self::Error> {
        match self {
            DomainType::BlkDeviceDomain(domain) => Ok(domain),
            DomainType::CacheBlkDeviceDomain(domain) => Ok(domain),
            DomainType::RtcDomain(domain) => Ok(domain),
            DomainType::GpuDomain(domain) => Ok(domain),
            DomainType::InputDomain(domain) => Ok(domain),
            DomainType::UartDomain(domain) => Ok(domain),
            DomainType::ShadowBlockDomain(domain) => Ok(domain),
            DomainType::BufUartDomain(domain) => Ok(domain),
            DomainType::BufInputDomain(domain) => Ok(domain),
            DomainType::NetDomain(domain) => Ok(domain),
            _ => Err(AlienError::EINVAL),
        }
    }
}

#[cfg(feature = "domain")]
mod __impl {
    use core::{hint::spin_loop, sync::atomic::AtomicBool};

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
