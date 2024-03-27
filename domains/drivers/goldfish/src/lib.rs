#![no_std]
#![forbid(unsafe_code)]
#![allow(unused)]
mod rtc;
extern crate alloc;

use crate::rtc::GoldFishRtc;
use alloc::sync::Arc;
use basic::io::SafeIORegion;
use basic::println;
use constants::AlienResult;
use interface::{Basic, DeviceBase, DeviceInfo, DevicesDomain, DomainType, RtcDomain, RtcTime};
use rref::{RRef, RRefVec};
use spin::Once;
use time::macros::offset;
use time::OffsetDateTime;

static RTC: Once<Arc<GoldFishRtc>> = Once::new();

#[derive(Debug)]
struct Rtc;

impl Basic for Rtc {}

impl DeviceBase for Rtc {
    fn handle_irq(&self) -> AlienResult<()> {
        unimplemented!()
    }
}

impl RtcDomain for Rtc {
    fn init(&self, device_info: &DeviceInfo) -> AlienResult<()> {
        let rtc_space = &device_info.address_range;
        println!("Rtc region: {:#x?}", rtc_space);
        let safe_region =
            SafeIORegion::new(rtc_space.start, rtc_space.end - rtc_space.start).unwrap();
        let rtc = Arc::new(GoldFishRtc::new(safe_region));
        println!("current time: {:?}", rtc);
        RTC.call_once(|| rtc);
        Ok(())
    }

    fn read_time(&self, mut time: RRef<RtcTime>) -> AlienResult<RRef<RtcTime>> {
        let rtc = RTC.get().unwrap();
        let time_stamp = rtc.read_raw_time();
        let t = rtc.read_time_fmt();
        *time = t;
        Ok(time)
    }
}

pub fn main() -> Arc<dyn RtcDomain> {
    Arc::new(Rtc)
}
