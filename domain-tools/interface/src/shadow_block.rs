use crate::DeviceBase;
use constants::AlienResult;
use rref::RRef;

pub trait ShadowBlockDomain: DeviceBase {
    fn init(&self, blk_domain: &str) -> AlienResult<()>;
    fn read_block(&self, block: u32, data: RRef<[u8; 512]>) -> AlienResult<RRef<[u8; 512]>>;
    fn write_block(&self, block: u32, data: &RRef<[u8; 512]>) -> AlienResult<usize>;
    fn get_capacity(&self) -> AlienResult<u64>;
    fn flush(&self) -> AlienResult<()>;
}
