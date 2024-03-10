#![no_std]
#![deny(unsafe_code)]
#![allow(unused)]
mod rtc;

extern crate alloc;
extern crate malloc;

use crate::rtc::GoldFishRtc;
use alloc::sync::Arc;
use interface::{Basic, DeviceBase, DeviceInfo, RtcDomain, RtcTime};
use libsyscall::{println, DeviceType};
use region::SafeIORegion;
use rref::{RRef, RRefVec, RpcResult};
use time::macros::offset;
use time::OffsetDateTime;

impl Basic for GoldFishRtc {}

impl DeviceBase for GoldFishRtc {
    fn handle_irq(&self) -> RpcResult<()> {
        unimplemented!()
    }
}

impl RtcDomain for GoldFishRtc {
    fn read_time(&self, mut time: RRef<RtcTime>) -> RpcResult<RRef<RtcTime>> {
        let time_stamp = self.read_raw_time();
        let t = self.read_time_fmt();
        *time = t;
        Ok(time)
    }
}

pub fn main() -> Arc<dyn RtcDomain> {
    let devices_domain = libsyscall::get_devices_domain().unwrap();
    let name = RRefVec::from_slice("rtc".as_bytes());

    let info = RRef::new(DeviceInfo {
        address_range: Default::default(),
        irq: RRef::new(0),
        compatible: RRefVec::new(0, 64),
    });

    let info = devices_domain.get_device(name, info).unwrap();

    let rtc_space = &info.address_range;

    println!("Rtc region: {:#x?}", rtc_space);
    let safe_region = SafeIORegion::new(rtc_space.start, rtc_space.end - rtc_space.start).unwrap();
    let rtc = Arc::new(GoldFishRtc::new(safe_region));
    println!("current time: {:?}", rtc);
    rtc
}
