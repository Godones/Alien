#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::boxed::Box;
use core::ops::Range;

use basic::{constants::io::RtcTime, io::SafeIORegion, println, AlienResult};
use interface::{Basic, DeviceBase, RtcDomain};
use rref::RRef;
use rtc::{goldfish::GoldFishRtc, LowRtcDevice, RtcIORegion};
use spin::Once;
use timestamp::DateTime;

static RTC: Once<GoldFishRtc> = Once::new();

#[derive(Debug)]
struct Rtc;

#[derive(Debug)]
pub struct SafeIORegionWrapper(SafeIORegion);

impl RtcIORegion for SafeIORegionWrapper {
    fn read_at(&self, offset: usize) -> u32 {
        self.0.read_at(offset).unwrap()
    }

    fn write_at(&self, offset: usize, value: u32) {
        self.0.write_at(offset, value).unwrap()
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
        let time_stamp_nanos = rtc.read_time();
        const NANOS_PER_SEC: usize = 1_000_000_000;
        let date = DateTime::new(time_stamp_nanos as usize / NANOS_PER_SEC);
        let t = RtcTime {
            year: date.year as u32,
            mon: date.month as u32,
            mday: date.day as u32,
            hour: date.hour as u32,
            min: date.minutes as u32,
            sec: date.seconds as u32,
            ..Default::default()
        };
        *time = t;
        Ok(time)
    }
}

pub fn main() -> Box<dyn RtcDomain> {
    Box::new(Rtc)
}
