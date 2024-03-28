use constants::AlienResult;
use rref::RRef;

use crate::{devices::DeviceInfo, DeviceBase};

pub trait BlkDeviceDomain: DeviceBase {
    fn init(&self, device_info: &DeviceInfo) -> AlienResult<()>;
    fn read_block(&self, block: u32, data: RRef<[u8; 512]>) -> AlienResult<RRef<[u8; 512]>>;
    fn write_block(&self, block: u32, data: &RRef<[u8; 512]>) -> AlienResult<usize>;
    fn get_capacity(&self) -> AlienResult<u64>;
    fn flush(&self) -> AlienResult<()>;
    fn restart(&self) -> bool {
        false
    }
}
