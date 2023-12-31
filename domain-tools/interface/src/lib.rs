#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use core::hint::spin_loop;
use core::sync::atomic::AtomicBool;
use rref::{RRef, RpcResult};

pub trait Basic {
    fn drop_self(self: Box<Self>) {
        drop(self);
    }
    fn is_active(&self) -> bool {
        is_active()
    }
}

#[cfg(feature = "blk")]
pub trait BlkDevice: Send + Sync + Basic {
    fn read(&mut self, block: u32, data: RRef<[u8; 512]>) -> RpcResult<RRef<[u8; 512]>>;
    fn write(&mut self, block: u32, data: &RRef<[u8; 512]>) -> RpcResult<usize>;
    fn get_capacity(&self) -> RpcResult<u64>;
    fn flush(&self) -> RpcResult<()>;
}

#[cfg(feature = "fs")]
pub trait Fs: Send + Sync + Basic {
    fn ls(&self, path: RRef<[u8; 512]>) -> RpcResult<RRef<[u8; 512]>>;
}

static ACTIVE: AtomicBool = AtomicBool::new(false);

/// Activate the domain
///
/// It should be called in the `main` function of the domain.
pub fn activate_domain() {
    ACTIVE.store(true, core::sync::atomic::Ordering::SeqCst);
}

fn is_active() -> bool {
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
