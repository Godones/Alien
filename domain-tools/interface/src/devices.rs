use crate::Basic;
use constants::AlienResult;
use core::ops::Range;
use rref::RRef;

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub address_range: Range<usize>,
    pub irq: u32,
    pub name: [u8; 64],
    pub next: usize,
}

impl Default for DeviceInfo {
    fn default() -> Self {
        Self {
            address_range: 0..0,
            irq: 0,
            name: [0; 64],
            next: 0,
        }
    }
}

pub trait DevicesDomain: Basic {
    fn init(&self, dtb: &'static [u8]) -> AlienResult<()>;
    /// if there is no device, the return value next is 0
    fn index_device(&self, index: usize, info: RRef<DeviceInfo>) -> AlienResult<RRef<DeviceInfo>>;
}
