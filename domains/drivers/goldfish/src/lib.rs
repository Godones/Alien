#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::boxed::Box;
use core::ops::Range;

use basic::{io::SafeIORegion, println};
use constants::{io::RtcTime, AlienResult};
use goldfish_rtc::GoldFishRtc;
use interface::{Basic, DeviceBase, RtcDomain};
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
    fn init(&self, address_range: Range<usize>) -> AlienResult<()> {
        println!("Rtc region: {:#x?}", address_range);
        let safe_region = SafeIORegion::from(address_range);
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
