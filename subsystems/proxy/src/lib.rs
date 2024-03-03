#![no_std]
#![feature(linkage)]
#![feature(naked_functions)]
extern crate alloc;

mod trampoline;

pub use trampoline::pop_continuation;

use alloc::sync::Arc;
use core::arch::asm;
use core::fmt::Debug;
use interface::{Basic, BlkDeviceDomain, FsDomain, RtcDomain, RtcTime};
use rref::{RRef, RpcError, RpcResult};

#[derive(Debug)]
pub struct BlkDomainProxy {
    domain_id: u64,
    domain: Arc<dyn BlkDeviceDomain>,
}

impl BlkDomainProxy {
    pub fn new(domain_id: u64, domain: Arc<dyn BlkDeviceDomain>) -> Self {
        Self { domain_id, domain }
    }
}

impl Basic for BlkDomainProxy {
    fn is_active(&self) -> bool {
        self.domain.is_active()
    }
}

impl BlkDeviceDomain for BlkDomainProxy {
    fn read_block(&self, block: u32, data: RRef<[u8; 512]>) -> RpcResult<RRef<[u8; 512]>> {
        if !self.domain.is_active() {
            return Err(RpcError::DomainCrash);
        }
        // trace!("BlkDomainProxy_read: block: {}", block);
        // self.domain.read(block, data)
        unsafe { blk_domain_proxy_read_trampoline(&self.domain, block, data) }
    }
    fn write_block(&self, block: u32, data: &RRef<[u8; 512]>) -> RpcResult<usize> {
        if !self.domain.is_active() {
            return Err(RpcError::DomainCrash);
        }
        self.domain.write_block(block, data)
    }
    fn get_capacity(&self) -> RpcResult<u64> {
        if !self.is_active() {
            return Err(RpcError::DomainCrash);
        }
        self.domain.get_capacity()
    }
    fn flush(&self) -> RpcResult<()> {
        if !self.is_active() {
            return Err(RpcError::DomainCrash);
        }
        self.domain.flush()
    }

    fn handle_irq(&self) -> RpcResult<()> {
        if !self.is_active() {
            return Err(RpcError::DomainCrash);
        }
        self.domain.handle_irq()
    }
}
#[naked]
#[no_mangle]
#[allow(undefined_naked_function_abi)]
unsafe fn blk_domain_proxy_read_trampoline(
    blk_domain: &Arc<dyn BlkDeviceDomain>,
    block: u32,
    data: RRef<[u8; 512]>,
) -> RpcResult<RRef<[u8; 512]>> {
    asm!(
        "addi sp, sp, -33*8",
        "sd x0, 0*8(sp)",
        "sd x1, 1*8(sp)",
        "sd x2, 2*8(sp)",
        "sd x3, 3*8(sp)",
        "sd x4, 4*8(sp)",
        "sd x5, 5*8(sp)",
        "sd x6, 6*8(sp)",
        "sd x7, 7*8(sp)",
        "sd x8, 8*8(sp)",
        "sd x9, 9*8(sp)",
        "sd x10, 10*8(sp)",
        "sd x11, 11*8(sp)",
        "sd x12, 12*8(sp)",
        "sd x13, 13*8(sp)",
        "sd x14, 14*8(sp)",
        "sd x15, 15*8(sp)",
        "sd x16, 16*8(sp)",
        "sd x17, 17*8(sp)",
        "sd x18, 18*8(sp)",
        "sd x19, 19*8(sp)",
        "sd x20, 20*8(sp)",
        "sd x21, 21*8(sp)",
        "sd x22, 22*8(sp)",
        "sd x23, 23*8(sp)",
        "sd x24, 24*8(sp)",
        "sd x25, 25*8(sp)",
        "sd x26, 26*8(sp)",
        "sd x27, 27*8(sp)",
        "sd x28, 28*8(sp)",
        "sd x29, 29*8(sp)",
        "sd x30, 30*8(sp)",
        "sd x31, 31*8(sp)",
        "call blk_domain_proxy_read_ptr",
        "sd a0, 32*8(sp)",
        "mv a0, sp",
        "call register_cont",
        //  recover caller saved registers
        "ld ra, 1*8(sp)",
        "ld x5, 5*8(sp)",
        "ld x6, 6*8(sp)",
        "ld x7, 7*8(sp)",
        "ld x10, 10*8(sp)",
        "ld x11, 11*8(sp)",
        "ld x12, 12*8(sp)",
        "ld x13, 13*8(sp)",
        "ld x14, 14*8(sp)",
        "ld x15, 15*8(sp)",
        "ld x16, 16*8(sp)",
        "ld x17, 17*8(sp)",
        "ld x28, 28*8(sp)",
        "ld x29, 29*8(sp)",
        "ld x30, 30*8(sp)",
        "ld x31, 31*8(sp)",
        "addi sp, sp, 33*8",
        "la gp, blk_domain_proxy_read",
        "jr gp",
        options(noreturn)
    )
}

#[no_mangle]
fn blk_domain_proxy_read(
    blk_domain: &Arc<dyn BlkDeviceDomain>,
    block: u32,
    data: RRef<[u8; 512]>,
) -> RpcResult<RRef<[u8; 512]>> {
    // info!("BlkDomainProxy_read");
    blk_domain.read_block(block, data)
}
#[no_mangle]
fn blk_domain_proxy_read_err(
    _blk_domain: &Arc<dyn BlkDeviceDomain>,
    _block: u32,
    _data: RRef<[u8; 512]>,
) -> RpcResult<RRef<[u8; 512]>> {
    platform::println!("BlkDomainProxy_read should return error");
    Err(RpcError::DomainCrash)
}

#[no_mangle]
fn blk_domain_proxy_read_ptr() -> usize {
    blk_domain_proxy_read_err as usize
}

#[derive(Debug)]
pub struct FsDomainProxy {
    domain_id: u64,
    domain: Arc<dyn FsDomain>,
}

impl FsDomainProxy {
    pub fn new(domain_id: u64, domain: Arc<dyn FsDomain>) -> Self {
        Self { domain_id, domain }
    }
}

impl Basic for FsDomainProxy {
    fn is_active(&self) -> bool {
        self.domain.is_active()
    }
}

impl FsDomain for FsDomainProxy {}

#[derive(Debug)]
pub struct RtcDomainProxy {
    domain: Arc<dyn RtcDomain>,
}

impl RtcDomainProxy {
    pub fn new(_domain_id: u64, domain: Arc<dyn RtcDomain>) -> Self {
        Self { domain }
    }
}

impl Basic for RtcDomainProxy {
    fn is_active(&self) -> bool {
        self.domain.is_active()
    }
}

impl RtcDomain for RtcDomainProxy {
    fn read_time(&self, time: RRef<RtcTime>) -> RpcResult<RRef<RtcTime>> {
        if self.domain.is_active() {
            self.domain.read_time(time)
        } else {
            Err(RpcError::DomainCrash)
        }
    }
    fn handle_irq(&self) -> RpcResult<()> {
        if self.domain.is_active() {
            self.domain.handle_irq()
        } else {
            Err(RpcError::DomainCrash)
        }
    }
}
