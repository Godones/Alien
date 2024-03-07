#![no_std]
extern crate alloc;
extern crate malloc;

use alloc::sync::Arc;
use interface::{Basic, BlkDeviceDomain};
use log::error;
use rref::{RRef, RpcError, RpcResult};

#[derive(Debug)]
pub struct ShadowBlockDomain {
    block_domain: Arc<dyn BlkDeviceDomain>,
}

impl ShadowBlockDomain {
    pub fn new(block_domain: Arc<dyn BlkDeviceDomain>) -> Self {
        Self { block_domain }
    }
}

impl Basic for ShadowBlockDomain {}

impl BlkDeviceDomain for ShadowBlockDomain {
    fn read_block(&self, block: u32, data: RRef<[u8; 512]>) -> RpcResult<RRef<[u8; 512]>> {
        let mut data = data;
        let res = self.block_domain.read_block(block, data);
        match res {
            Ok(res) => Ok(res),
            Err(RpcError::DomainCrash) => {
                error!("domain crash, try restart domain");
                // try restart domain once
                if self.block_domain.restart() {
                    error!("restart domain ok");
                    data = RRef::new([0u8; 512]);
                    self.block_domain.read_block(block, data)
                } else {
                    Err(RpcError::DomainCrash)
                }
            }
            Err(e) => Err(e),
        }
    }

    fn write_block(&self, block: u32, data: &RRef<[u8; 512]>) -> RpcResult<usize> {
        self.block_domain.write_block(block, data)
    }

    fn get_capacity(&self) -> RpcResult<u64> {
        self.block_domain.get_capacity()
    }

    fn flush(&self) -> RpcResult<()> {
        self.block_domain.flush()
    }

    fn handle_irq(&self) -> RpcResult<()> {
        self.block_domain.handle_irq()
    }
}

pub fn main() -> Arc<dyn BlkDeviceDomain> {
    let blk = libsyscall::get_blk_domain().unwrap();
    Arc::new(ShadowBlockDomain::new(blk))
}
