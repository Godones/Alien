//! compatible = "googlRtcDevicee,goldfish-rtc";

use alloc::format;
use basic::io::SafeIORegion;
use core::fmt::{Debug, Formatter};
use interface::{RtcDomain, RtcTime};
use time::macros::offset;
use time::OffsetDateTime;

const RTC_TIME_LOW: usize = 0x00;
const RTC_TIME_HIGH: usize = 0x04;
const RTC_ALARM_LOW: usize = 0x08;
const RTC_ALARM_HIGH: usize = 0x0c;
const RTC_IRQ_ENABLED: usize = 0x10;
const RTC_CLEAR_ALARM: usize = 0x14;
const RTC_ALARM_STATUS: usize = 0x18;
const RTC_CLEAR_INTERRUPT: usize = 0x1c;

pub struct GoldFishRtc {
    region: SafeIORegion,
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
    pub fn new(region: SafeIORegion) -> Self {
        let rtc = Self { region };
        rtc
    }
}

impl GoldFishRtc {
    pub fn read_raw_time(&self) -> u64 {
        // let time_low_addr = self.base_addr + RTC_TIME_LOW;
        // let time_high_addr = self.base_addr + RTC_TIME_HIGH;
        // let time_low = read_u32(time_low_addr);
        // let time_high = read_u32(time_high_addr);

        let time_low = self.region.read_at::<u32>(RTC_TIME_LOW).unwrap();
        let time_high = self.region.read_at::<u32>(RTC_TIME_HIGH).unwrap();

        (time_high as u64) << 32 | time_low as u64
    }

    pub fn read_time_fmt(&self) -> RtcTime {
        let time_stamp = self.read_raw_time();
        let t =
            OffsetDateTime::from_unix_timestamp_nanos(time_stamp as i128).expect("invalid time");
        let t = t.to_offset(offset!(+8));
        RtcTime {
            year: t.year() as u32,
            mon: t.month() as u32,
            mday: t.day() as u32,
            hour: t.hour() as u32,
            min: t.minute() as u32,
            sec: t.second() as u32,
            ..Default::default()
        }
    }

    fn set_time(&self, time: u64) {
        // let time_low_addr = self.base_addr + RTC_TIME_LOW;
        // let time_high_addr = self.base_addr + RTC_TIME_HIGH;
        let time_low = time as u32;
        let time_high = (time >> 32) as u32;
        // write_u32(time_low_addr, time_low);
        // write_u32(time_high_addr, time_high);
        self.region.write_at::<u32>(RTC_TIME_LOW, time_low).unwrap();
        self.region
            .write_at::<u32>(RTC_TIME_HIGH, time_high)
            .unwrap();
    }

    fn enable_irq(&self) {
        // let irq_enabled_addr = self.base_addr + RTC_IRQ_ENABLED;
        // write_u32(irq_enabled_addr, 1);
        self.region.write_at::<u32>(RTC_IRQ_ENABLED, 1).unwrap();
    }

    fn disable_irq(&self) {
        // let irq_enabled_addr = self.base_addr + RTC_IRQ_ENABLED;
        // write_u32(irq_enabled_addr, 0);
        self.region.write_at::<u32>(RTC_IRQ_ENABLED, 0).unwrap();
    }

    fn clear_irq(&self) {
        // let clear_irq_addr = self.base_addr + RTC_CLEAR_INTERRUPT;
        // write_u32(clear_irq_addr, 1);
        self.region.write_at::<u32>(RTC_CLEAR_INTERRUPT, 1).unwrap();
    }

    fn read_alarm(&self) -> u64 {
        // let alarm_low_addr = self.base_addr + RTC_ALARM_LOW;
        // let alarm_high_addr = self.base_addr + RTC_ALARM_HIGH;
        // let alarm_low = read_u32(alarm_low_addr);
        // let alarm_high = read_u32(alarm_high_addr);
        let alarm_low = self.region.read_at::<u32>(RTC_ALARM_LOW).unwrap();
        let alarm_high = self.region.read_at::<u32>(RTC_ALARM_HIGH).unwrap();
        (alarm_high as u64) << 32 | alarm_low as u64
    }

    fn set_alarm(&self, time: u64) {
        // let alarm_low_addr = self.base_addr + RTC_ALARM_LOW;
        // let alarm_high_addr = self.base_addr + RTC_ALARM_HIGH;
        let alarm_low = time as u32;
        let alarm_high = (time >> 32) as u32;
        // write_u32(alarm_low_addr, alarm_low);
        // write_u32(alarm_high_addr, alarm_high);
        self.region
            .write_at::<u32>(RTC_ALARM_LOW, alarm_low)
            .unwrap();
        self.region
            .write_at::<u32>(RTC_ALARM_HIGH, alarm_high)
            .unwrap();
    }

    fn clear_alarm(&self) {
        // let clear_alarm_addr = self.base_addr + RTC_CLEAR_ALARM;
        // write_u32(clear_alarm_addr, 1);
        self.region.write_at::<u32>(RTC_CLEAR_ALARM, 1).unwrap();
    }

    fn alarm_status(&self) -> bool {
        // let alarm_status_addr = self.base_addr + RTC_ALARM_STATUS;
        // let alarm_status = read_u32(alarm_status_addr);
        let alarm_status = self.region.read_at::<u32>(RTC_ALARM_STATUS).unwrap();
        alarm_status == 1
    }

    fn is_irq_enabled(&self) -> bool {
        // let irq_enabled_addr = self.base_addr + RTC_IRQ_ENABLED;
        // let irq_enabled = read_u32(irq_enabled_addr);
        let irq_enabled = self.region.read_at::<u32>(RTC_IRQ_ENABLED).unwrap();
        irq_enabled == 1
    }
}
