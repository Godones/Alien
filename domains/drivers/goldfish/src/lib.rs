#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::boxed::Box;
use core::ops::Range;

use basic::{io::SafeIORegion, println};
use constants::{io::RtcTime, AlienResult};
use goldfish_rtc::{GoldFishRtc, GoldFishRtcIo};
use interface::{Basic, DeviceBase, RtcDomain};
use rref::RRef;
use spin::Once;

static RTC: Once<GoldFishRtc> = Once::new();

#[derive(Debug)]
struct Rtc;

#[derive(Debug)]
pub struct SafeIORegionWrapper(SafeIORegion);

impl GoldFishRtcIo for SafeIORegionWrapper {
    fn read_at(&self, offset: usize) -> AlienResult<u32> {
        self.0.read_at(offset)
    }

    fn write_at(&self, offset: usize, value: u32) -> AlienResult<()> {
        self.0.write_at(offset, value)
    }
}

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
        let rtc = GoldFishRtc::new(Box::new(SafeIORegionWrapper(safe_region)));
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
