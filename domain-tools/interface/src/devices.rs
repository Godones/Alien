use crate::Basic;
use core::ops::Range;
use rref::{RRef, RRefVec};

#[derive(Debug)]
pub struct DeviceInfo {
    pub address_range: Range<usize>,
    pub irq: RRef<u32>,
    pub compatible: RRefVec<u8>,
}

pub trait DevicesDomain: Basic {
    fn get_device(&self, name: RRefVec<u8>, info: RRef<DeviceInfo>) -> Option<RRef<DeviceInfo>>;
}
