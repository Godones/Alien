#![forbid(unsafe_code)]
#![allow(unused)]
#![cfg_attr(not(test), no_std)]
//! compatible = "googlRtcDevicee,goldfish-rtc";
extern crate alloc;

use alloc::boxed::Box;
use core::fmt::{Debug, Formatter};

use constants::{io::RtcTime, AlienResult};
use log::info;
use timestamp::DateTime;

const RTC_TIME_LOW: usize = 0x00;
const RTC_TIME_HIGH: usize = 0x04;
const RTC_ALARM_LOW: usize = 0x08;
const RTC_ALARM_HIGH: usize = 0x0c;
const RTC_IRQ_ENABLED: usize = 0x10;
const RTC_CLEAR_ALARM: usize = 0x14;
const RTC_ALARM_STATUS: usize = 0x18;
const RTC_CLEAR_INTERRUPT: usize = 0x1c;
const NANOS_PER_SEC: usize = 1_000_000_000;

pub trait GoldFishRtcIo: Debug + Send + Sync {
    fn read_at(&self, offset: usize) -> AlienResult<u32>;
    fn write_at(&self, offset: usize, value: u32) -> AlienResult<()>;
}

pub struct GoldFishRtc {
    region: Box<dyn GoldFishRtcIo>,
}

impl Debug for GoldFishRtc {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let time = self.read_time_fmt();
        f.write_fmt(format_args!(
            "{}-{}-{} {}:{}:{}",
            time.year, time.mon, time.mday, time.hour, time.min, time.sec
        ))
    }
}

impl GoldFishRtc {
    pub fn new(region: Box<dyn GoldFishRtcIo>) -> Self {
        let rtc = Self { region };
        rtc
    }
}

impl GoldFishRtc {
    pub fn read_raw_time(&self) -> u64 {
        let time_low = self.region.read_at(RTC_TIME_LOW).unwrap();
        let time_high = self.region.read_at(RTC_TIME_HIGH).unwrap();
        (time_high as u64) << 32 | time_low as u64
    }

    pub fn read_time_fmt(&self) -> RtcTime {
        let time_stamp_nanos = self.read_raw_time();
        let date = DateTime::new(time_stamp_nanos as usize / NANOS_PER_SEC);
        RtcTime {
            year: date.year as u32,
            mon: date.month as u32,
            mday: date.day as u32,
            hour: date.hour as u32,
            min: date.minutes as u32,
            sec: date.seconds as u32,
            ..Default::default()
        }
    }

    fn set_time(&self, time: u64) {
        let time_low = time as u32;
        let time_high = (time >> 32) as u32;
        self.region.write_at(RTC_TIME_LOW, time_low).unwrap();
        self.region.write_at(RTC_TIME_HIGH, time_high).unwrap();
    }

    fn enable_irq(&self) {
        self.region.write_at(RTC_IRQ_ENABLED, 1).unwrap();
    }

    fn disable_irq(&self) {
        self.region.write_at(RTC_IRQ_ENABLED, 0).unwrap();
    }

    fn clear_irq(&self) {
        self.region.write_at(RTC_CLEAR_INTERRUPT, 1).unwrap();
    }

    fn read_alarm(&self) -> u64 {
        let alarm_low = self.region.read_at(RTC_ALARM_LOW).unwrap();
        let alarm_high = self.region.read_at(RTC_ALARM_HIGH).unwrap();
        (alarm_high as u64) << 32 | alarm_low as u64
    }

    fn set_alarm(&self, time: u64) {
        let alarm_low = time as u32;
        let alarm_high = (time >> 32) as u32;
        self.region.write_at(RTC_ALARM_LOW, alarm_low).unwrap();
        self.region.write_at(RTC_ALARM_HIGH, alarm_high).unwrap();
    }

    fn clear_alarm(&self) {
        self.region.write_at(RTC_CLEAR_ALARM, 1).unwrap();
    }

    fn alarm_status(&self) -> bool {
        let alarm_status = self.region.read_at(RTC_ALARM_STATUS).unwrap();
        alarm_status == 1
    }

    fn is_irq_enabled(&self) -> bool {
        let irq_enabled = self.region.read_at(RTC_IRQ_ENABLED).unwrap();
        irq_enabled == 1
    }
}
