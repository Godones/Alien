use constants::AlienResult;
use downcast_rs::{impl_downcast, DowncastSync};
use gproxy::proxy;
use rref::RRef;

use crate::{Basic, DeviceBase};

#[proxy(ShadowBlockDomainProxy)]
pub trait ShadowBlockDomain: DeviceBase + Basic + DowncastSync {
    fn init(&self, blk_domain: &str) -> AlienResult<()>;
    fn read_block(&self, block: u32, data: RRef<[u8; 512]>) -> AlienResult<RRef<[u8; 512]>>;
    fn write_block(&self, block: u32, data: &RRef<[u8; 512]>) -> AlienResult<usize>;
    fn get_capacity(&self) -> AlienResult<u64>;
    fn flush(&self) -> AlienResult<()>;
}

impl_downcast!(sync  ShadowBlockDomain);
