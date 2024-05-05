use alloc::{format, string::String};

use constants::io::RtcTime;
use device_interface::{DeviceBase, RtcDevice};
use log::trace;
use rtc::{LowRtcDevice, LowRtcDeviceExt};

type GFish = rtc::goldfish::GoldFishRtc;
pub struct GoldFishRtc {
    gold_fish_rtc: GFish,
}

impl GoldFishRtc {
    pub fn read_time_u64(&self) -> u64 {
        self.gold_fish_rtc.read_time()
    }
    pub fn read_time_string(&self) -> String {
        let time = self.gold_fish_rtc.read_time_fmt();
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            time.year, time.month, time.day, time.hour, time.minute, time.second
        )
    }
}

impl DeviceBase for GoldFishRtc {
    fn handle_irq(&self) {
        let alarm = self.gold_fish_rtc.read_alarm_fmt();
        let time = self.gold_fish_rtc.read_time_fmt();
        trace!("rtc interrupt, time: {:?}, alarm: {:?}", time, alarm);
        self.gold_fish_rtc.clear_irq();
        self.gold_fish_rtc
            .set_alarm(self.read_time_u64() + 1_000_000_000 * 5);
    }
}

impl RtcDevice for GoldFishRtc {
    fn read_time(&self) -> RtcTime {
        let time = self.gold_fish_rtc.read_time_fmt();
        RtcTime {
            year: time.year,
            wday: 0,
            yday: 0,
            mon: time.month as u32,
            mday: time.day as u32,
            hour: time.hour as u32,
            min: time.minute as u32,
            sec: time.second as u32,
            isdst: 0,
        }
    }
}

impl GoldFishRtc {
    pub fn new(base_addr: usize) -> Self {
        let gold_fish_rtc = GFish::new(base_addr);
        Self { gold_fish_rtc }
    }
}
