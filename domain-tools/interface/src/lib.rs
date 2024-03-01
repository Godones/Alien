#![no_std]

extern crate alloc;

use core::any::Any;
use core::fmt::Debug;
use rref::{RRef, RpcResult};

pub trait Basic: Any {
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

#[cfg(feature = "blk")]
pub trait BlkDevice: Send + Sync + Basic + Debug {
    fn read(&self, block: u32, data: RRef<[u8; 512]>) -> RpcResult<RRef<[u8; 512]>>;
    fn write(&self, block: u32, data: &RRef<[u8; 512]>) -> RpcResult<usize>;
    fn get_capacity(&self) -> RpcResult<u64>;
    fn flush(&self) -> RpcResult<()>;
    fn handle_irq(&self) -> RpcResult<()>;
}

#[cfg(feature = "fs")]
pub trait Fs: Send + Sync + Basic + Debug {
    fn ls(&self, path: RRef<[u8; 512]>) -> RpcResult<RRef<[u8; 512]>>;
}

#[cfg(feature = "uart")]
pub trait Uart: Send + Sync + Basic + Debug {
    /// Write a character to the UART
    fn putc(&self, ch: u8) -> RpcResult<()>;
    /// Read a character from the UART
    fn getc(&self) -> RpcResult<Option<u8>>;
    fn handle_irq(&self) -> RpcResult<()>;
}

#[cfg(feature = "gpu")]
pub trait Gpu: Send + Sync + Basic + Debug {
    fn flush(&self) -> RpcResult<()>;
    fn fill_buf(&self, buf: RRef<[u8; 1280 * 800]>) -> RpcResult<()>;
    fn handle_irq(&self) -> RpcResult<()>;
}

#[cfg(feature = "input")]
pub trait Input: Send + Sync + Basic + Debug {
    /// Read an input event from the input device
    fn event(&self) -> RpcResult<Option<u64>>;
    fn handle_irq(&self) -> RpcResult<()>;
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
