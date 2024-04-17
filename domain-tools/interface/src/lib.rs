#![no_std]

mod block;
mod buf_input;
mod buf_uart;
mod cache_block;
mod empty_device;
mod fs;
mod gpu;
mod input_device;
mod net;
mod plic;
mod rtc;
mod scheduler;
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

pub trait DeviceBase: Send + Sync {
    fn handle_irq(&self) -> AlienResult<()>;
}

pub use block::*;
pub use buf_input::*;
pub use buf_uart::*;
pub use cache_block::*;
pub use empty_device::*;
pub use fs::*;
pub use gpu::*;
pub use input_device::*;
pub use net::*;
pub use plic::*;
pub use rtc::*;
pub use scheduler::*;
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
    TaskDomain(Arc<dyn TaskDomain>),
    SysCallDomain(Arc<dyn SysCallDomain>),
    ShadowBlockDomain(Arc<dyn ShadowBlockDomain>),
    BufUartDomain(Arc<dyn BufUartDomain>),
    NetDeviceDomain(Arc<dyn NetDomain>),
    BufInputDomain(Arc<dyn BufInputDomain>),
    EmptyDeviceDomain(Arc<dyn EmptyDeviceDomain>),
    DevFsDomain(Arc<dyn DevFsDomain>),
    SchedulerDomain(Arc<dyn SchedulerDomain>),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainTypeRaw {
    FsDomain = 1,
    BlkDeviceDomain = 2,
    CacheBlkDeviceDomain = 3,
    RtcDomain = 4,
    GpuDomain = 5,
    InputDomain = 6,
    VfsDomain = 7,
    UartDomain = 8,
    PLICDomain = 9,
    TaskDomain = 10,
    SysCallDomain = 11,
    ShadowBlockDomain = 12,
    BufUartDomain = 13,
    NetDeviceDomain = 14,
    BufInputDomain = 15,
    EmptyDeviceDomain = 16,
    DevFsDomain = 17,
    SchedulerDomain = 18,
}

impl TryFrom<u8> for DomainTypeRaw {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(DomainTypeRaw::FsDomain),
            2 => Ok(DomainTypeRaw::BlkDeviceDomain),
            3 => Ok(DomainTypeRaw::CacheBlkDeviceDomain),
            4 => Ok(DomainTypeRaw::RtcDomain),
            5 => Ok(DomainTypeRaw::GpuDomain),
            6 => Ok(DomainTypeRaw::InputDomain),
            7 => Ok(DomainTypeRaw::VfsDomain),
            8 => Ok(DomainTypeRaw::UartDomain),
            9 => Ok(DomainTypeRaw::PLICDomain),
            10 => Ok(DomainTypeRaw::TaskDomain),
            11 => Ok(DomainTypeRaw::SysCallDomain),
            12 => Ok(DomainTypeRaw::ShadowBlockDomain),
            13 => Ok(DomainTypeRaw::BufUartDomain),
            14 => Ok(DomainTypeRaw::NetDeviceDomain),
            15 => Ok(DomainTypeRaw::BufInputDomain),
            16 => Ok(DomainTypeRaw::EmptyDeviceDomain),
            17 => Ok(DomainTypeRaw::DevFsDomain),
            18 => Ok(DomainTypeRaw::SchedulerDomain),
            _ => Err(()),
        }
    }
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
