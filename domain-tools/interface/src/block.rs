use core::ops::Range;

use constants::AlienResult;
use rref::RRef;

use crate::{Basic, DeviceBase};

pub trait BlkDeviceDomain: DeviceBase + Basic {
    fn init(&self, device_info: Range<usize>) -> AlienResult<()>;
    fn read_block(&self, block: u32, data: RRef<[u8; 512]>) -> AlienResult<RRef<[u8; 512]>>;
    fn write_block(&self, block: u32, data: &RRef<[u8; 512]>) -> AlienResult<usize>;
    fn get_capacity(&self) -> AlienResult<u64>;
    fn flush(&self) -> AlienResult<()>;
    fn restart(&self) -> bool {
        false
    }
}
