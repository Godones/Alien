#![no_std]

mod block;
mod cache_block;
mod devices;
mod gpu;
mod input_device;
mod plic;
mod rtc;
mod syscall;
mod task;
mod uart;
mod vfs;

extern crate alloc;

use core::any::Any;
use core::fmt::Debug;
use rref::RpcResult;

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
    fn handle_irq(&self) -> RpcResult<()>;
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
