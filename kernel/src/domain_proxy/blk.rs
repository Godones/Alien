use alloc::boxed::Box;
use core::{arch::asm, ops::Range};

use constants::{AlienError, AlienResult};
use interface::{Basic, BlkDeviceDomain, DeviceBase};
use ksync::{Mutex, RwLock};
use log::error;
use rref::RRef;

use crate::{domain_loader::loader::DomainLoader, domain_proxy::continuation};

#[derive(Debug)]
pub struct BlkDomainProxy {
    domain_id: u64,
    domain: RwLock<Box<dyn BlkDeviceDomain>>,
    domain_loader: DomainLoader,
    device_info: Mutex<Option<Range<usize>>>,
}

impl BlkDomainProxy {
    pub fn new(
        domain_id: u64,
        domain: Box<dyn BlkDeviceDomain>,
        domain_loader: DomainLoader,
    ) -> Self {
        Self {
            domain_id,
            domain: RwLock::new(domain),
            domain_loader,
            device_info: Mutex::new(None),
        }
    }
}

impl Basic for BlkDomainProxy {
    fn is_active(&self) -> bool {
        self.domain.read().is_active()
    }
}

impl DeviceBase for BlkDomainProxy {
    fn handle_irq(&self) -> AlienResult<()> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.read().handle_irq()
    }
}

impl BlkDeviceDomain for BlkDomainProxy {
    fn init(&self, device_info: Range<usize>) -> AlienResult<()> {
        self.device_info.lock().replace(device_info.clone());
        self.domain.read().init(device_info)
    }

    fn read_block(&self, block: u32, data: RRef<[u8; 512]>) -> AlienResult<RRef<[u8; 512]>> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        // self.domain.read().read_block(block, data)
        let res = {
            let guard = self.domain.read();
            unsafe { blk_domain_proxy_read_trampoline(&guard, block, data) }
        };
        res
    }
    fn write_block(&self, block: u32, data: &RRef<[u8; 512]>) -> AlienResult<usize> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.read().write_block(block, data)
    }
    fn get_capacity(&self) -> AlienResult<u64> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.read().get_capacity()
    }
    fn flush(&self) -> AlienResult<()> {
        if !self.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.read().flush()
    }

    // todo!()
    fn restart(&self) -> bool {
        let mut domain = self.domain.write();
        self.domain_loader.reload().unwrap();
        // let mut loader = DomainLoader::new(self.domain_loader.data());
        // loader.load().unwrap();
        // let new_domain = loader.call(self.domain_id);
        let mut new_domain = self
            .domain_loader
            .call::<dyn BlkDeviceDomain>(self.domain_id);
        let device_info = self.device_info.lock();
        new_domain
            .init(device_info.as_ref().unwrap().clone())
            .unwrap();
        core::mem::swap(&mut *domain, &mut new_domain);
        // The new_domain now is the old domain, but it has been recycled so we
        // can't drop it again
        core::mem::forget(new_domain);
        true
    }
}
#[naked]
#[no_mangle]
#[allow(undefined_naked_function_abi)]
unsafe fn blk_domain_proxy_read_trampoline(
    blk_domain: &Box<dyn BlkDeviceDomain>,
    block: u32,
    data: RRef<[u8; 512]>,
) -> AlienResult<RRef<[u8; 512]>> {
    asm!(
        "addi sp, sp, -33*8",
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
    blk_domain: &Box<dyn BlkDeviceDomain>,
    block: u32,
    data: RRef<[u8; 512]>,
) -> AlienResult<RRef<[u8; 512]>> {
    // info!("BlkDomainProxy_read");
    let res = blk_domain.read_block(block, data);
    continuation::pop_continuation();
    res
}
#[no_mangle]
fn blk_domain_proxy_read_err() -> AlienResult<RRef<[u8; 512]>> {
    error!("BlkDomainProxy_read should return error");
    Err(AlienError::DOMAINCRASH)
}

#[no_mangle]
fn blk_domain_proxy_read_ptr() -> usize {
    blk_domain_proxy_read_err as usize
}
