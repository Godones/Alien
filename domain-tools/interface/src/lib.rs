#![no_std]

mod block;
mod buf_input;
mod buf_uart;
mod cache_block;
mod devices;
mod empty_device;
mod fs;
mod gpu;
mod input_device;
mod net;
mod plic;
mod rtc;
mod sd;
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
    #[cfg(feature = "domain")]
    fn is_active(&self) -> bool {
        __impl::is_active()
    }
    #[cfg(not(feature = "domain"))]
    fn is_active(&self) -> bool {
        false
    }
}

pub trait DeviceBase {
    fn handle_irq(&self) -> AlienResult<()>;
}

pub use block::*;
pub use buf_input::*;
pub use buf_uart::*;
pub use cache_block::*;
pub use devices::{DeviceInfo, *};
pub use empty_device::*;
pub use fs::*;
pub use gpu::*;
pub use input_device::*;
pub use net::*;
pub use plic::*;
pub use rtc::*;
pub use shadow_block::*;
pub use syscall::*;
pub use task::*;
pub use uart::*;
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
    NetDeviceDomain(Arc<dyn NetDomain>),
    BufInputDomain(Arc<dyn BufInputDomain>),
    EmptyDeviceDomain(Arc<dyn EmptyDeviceDomain>),
    DevFsDomain(Arc<dyn DevFsDomain>),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainTypeRaw {
    FsDomain,
    BlkDeviceDomain,
    CacheBlkDeviceDomain,
    RtcDomain,
    GpuDomain,
    InputDomain,
    VfsDomain,
    UartDomain,
    PLICDomain,
    DevicesDomain,
    TaskDomain,
    SysCallDomain,
    ShadowBlockDomain,
    BufUartDomain,
    NetDeviceDomain,
    BufInputDomain,
    EmptyDeviceDomain,
    DevFsDomain,
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
            DomainType::NetDeviceDomain(domain) => Ok(domain),
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
