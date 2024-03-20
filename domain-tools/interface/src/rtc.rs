use crate::{DeviceBase, DeviceInfo};
use constants::AlienResult;
use rref::RRef;

#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
pub struct RtcTime {
    pub sec: u32,
    pub min: u32,
    pub hour: u32,
    pub mday: u32,
    pub mon: u32,
    pub year: u32,
    pub wday: u32,  // unused
    pub yday: u32,  // unused
    pub isdst: u32, // unused
}

pub trait RtcDomain: DeviceBase {
    fn init(&self, device_info: &DeviceInfo) -> AlienResult<()>;
    fn read_time(&self, time: RRef<RtcTime>) -> AlienResult<RRef<RtcTime>>;
}
