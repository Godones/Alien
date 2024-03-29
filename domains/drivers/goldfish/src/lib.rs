#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::boxed::Box;

use basic::{io::SafeIORegion, println};
use constants::{io::RtcTime, AlienResult};
use goldfish_rtc::GoldFishRtc;
use interface::{Basic, DeviceBase, DeviceInfo, RtcDomain};
use rref::RRef;
use spin::Once;

static RTC: Once<GoldFishRtc> = Once::new();

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
        let rtc = GoldFishRtc::new(Box::new(safe_region));
        println!("current time: {:?}", rtc);
        RTC.call_once(|| rtc);
        Ok(())
    }

    fn read_time(&self, mut time: RRef<RtcTime>) -> AlienResult<RRef<RtcTime>> {
        let rtc = RTC.get().unwrap();
        let t = rtc.read_time_fmt();
        *time = t;
        Ok(time)
    }
}

pub fn main() -> Box<dyn RtcDomain> {
    Box::new(Rtc)
}
