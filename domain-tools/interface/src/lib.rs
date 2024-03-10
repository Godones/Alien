#![no_std]

extern crate alloc;

use alloc::sync::Arc;
use core::any::Any;
use core::fmt::Debug;
use core::ops::Range;
use rref::{RRef, RRefVec, RpcResult};

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

#[cfg(feature = "blk")]
pub trait BlkDeviceDomain: DeviceBase {
    fn read_block(&self, block: u32, data: RRef<[u8; 512]>) -> RpcResult<RRef<[u8; 512]>>;
    fn write_block(&self, block: u32, data: &RRef<[u8; 512]>) -> RpcResult<usize>;
    fn get_capacity(&self) -> RpcResult<u64>;
    fn flush(&self) -> RpcResult<()>;
    fn restart(&self) -> bool {
        false
    }
}

#[cfg(feature = "cache_blk")]
pub trait CacheBlkDeviceDomain: DeviceBase {
    fn read(&self, offset: u64, buf: RRefVec<u8>) -> RpcResult<RRefVec<u8>>;
    fn write(&self, offset: u64, buf: &RRefVec<u8>) -> RpcResult<usize>;
    fn get_capacity(&self) -> RpcResult<u64>;
    fn flush(&self) -> RpcResult<()>;
}

#[cfg(feature = "fs")]
pub trait FsDomain: Basic {}

#[cfg(feature = "uart")]
pub trait UartDomain: DeviceBase {
    /// Write a character to the UART
    fn putc(&self, ch: u8) -> RpcResult<()>;
    /// Read a character from the UART
    fn getc(&self) -> RpcResult<Option<u8>>;
    /// Check if there is data to get from the UART
    fn have_data_to_get(&self) -> bool;
    /// Check if there is space to put data to the UART
    fn have_space_to_put(&self) -> bool {
        true
    }
}

#[cfg(feature = "gpu")]
pub trait GpuDomain: DeviceBase {
    fn flush(&self) -> RpcResult<()>;
    fn fill(&self, offset: u32, buf: &RRefVec<u8>) -> RpcResult<usize>;
}

#[cfg(feature = "input")]
pub trait InputDomain: DeviceBase {
    /// Read an input event from the input device
    fn event(&self) -> RpcResult<Option<u64>>;
}

#[cfg(feature = "vfs")]
pub trait VfsDomain: Basic {}

#[cfg(feature = "rtc")]
#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
pub struct RtcTime {
    pub sec: u32,
    pub min: u32,
    pub hour: u32,
    pub mday: u32,
    pub mon: u32,
    pub year: u32,
    pub wday: u32,  // unused
    pub yday: u32,  // unused
    pub isdst: u32, // unused
}

#[cfg(feature = "rtc")]
pub trait RtcDomain: DeviceBase {
    fn read_time(&self, time: RRef<RtcTime>) -> RpcResult<RRef<RtcTime>>;
}

#[cfg(feature = "plic")]
pub trait PLICDomain: Basic {
    fn handle_irq(&self) -> RpcResult<()>;
    fn register_irq(&self, irq: usize, device: Arc<dyn DeviceBase>) -> RpcResult<()>;
    fn irq_info(&self, buf: RRefVec<u8>) -> RpcResult<RRefVec<u8>>;
}

#[cfg(feature = "devices")]
#[derive(Debug)]
pub struct DeviceInfo {
    pub address_range: Range<usize>,
    pub irq: RRef<u32>,
    pub compatible: RRefVec<u8>,
}

#[cfg(feature = "devices")]
pub trait DevicesDomain: Basic {
    fn get_device(&self, name: RRefVec<u8>, info: RRef<DeviceInfo>) -> Option<RRef<DeviceInfo>>;
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
