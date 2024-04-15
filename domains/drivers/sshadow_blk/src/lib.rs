#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::{boxed::Box, sync::Arc};
use core::sync::atomic::AtomicBool;

use constants::{AlienError, AlienResult};
use interface::{Basic, BlkDeviceDomain, DeviceBase, DomainType, ShadowBlockDomain};
use log::error;
use rref::RRef;
use spin::Once;

static BLOCK: Once<Arc<dyn BlkDeviceDomain>> = Once::new();

#[derive(Debug)]
pub struct ShadowBlockDomainImpl;

impl ShadowBlockDomainImpl {
    pub fn new() -> Self {
        Self
    }
}

impl Basic for ShadowBlockDomainImpl {}

impl DeviceBase for ShadowBlockDomainImpl {
    fn handle_irq(&self) -> AlienResult<()> {
        BLOCK.get().unwrap().handle_irq()
    }
}

impl ShadowBlockDomain for ShadowBlockDomainImpl {
    fn init(&self, blk_domain: &str) -> AlienResult<()> {
        let blk = basic::get_domain(blk_domain).unwrap();
        let blk = match blk {
            DomainType::BlkDeviceDomain(blk) => blk,
            _ => panic!("not a block domain"),
        };
        BLOCK.call_once(|| blk);
        Ok(())
    }

    fn read_block(&self, block: u32, data: RRef<[u8; 512]>) -> AlienResult<RRef<[u8; 512]>> {
        static FLAG: AtomicBool = AtomicBool::new(false);
        if !FLAG.load(core::sync::atomic::Ordering::Relaxed) {
            error!("<SShadowBlockDomainImpl> read block: {}", block);
            FLAG.store(true, core::sync::atomic::Ordering::Relaxed);
        }
        let blk = BLOCK.get().unwrap();
        let mut data = data;
        let res = blk.read_block(block, data);
        match res {
            Ok(res) => Ok(res),
            Err(AlienError::DOMAINCRASH) => {
                error!("domain crash, try restart domain");
                // try restart domain once
                if blk.restart() {
                    error!("restart domain ok");
                    data = RRef::new([0u8; 512]);
                    blk.read_block(block, data)
                } else {
                    error!("restart domain failed");
                    Err(AlienError::DOMAINCRASH)
                }
            }
            Err(e) => Err(e),
        }
    }

    fn write_block(&self, block: u32, data: &RRef<[u8; 512]>) -> AlienResult<usize> {
        BLOCK.get().unwrap().write_block(block, data)
    }

    fn get_capacity(&self) -> AlienResult<u64> {
        BLOCK.get().unwrap().get_capacity()
    }

    fn flush(&self) -> AlienResult<()> {
        BLOCK.get().unwrap().flush()
    }
}

pub fn main() -> Box<dyn ShadowBlockDomain> {
    Box::new(ShadowBlockDomainImpl)
}
