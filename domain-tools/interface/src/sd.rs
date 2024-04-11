use constants::AlienResult;
use gproxy::proxy;
use rref::RRef;

use crate::{Basic, DeviceBase};

#[proxy(SDCardProxy)]
pub trait SDCard: Basic + DeviceBase {
    fn init(&self) -> AlienResult<()>;
    fn read_block(&self, block: u32, data: RRef<[u8; 512]>) -> AlienResult<RRef<[u8; 512]>>;
}
