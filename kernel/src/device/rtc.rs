use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use core::fmt::{Debug, Formatter};

use spin::Once;

use crate::interrupt::DeviceBase;

#[derive(Copy, Clone)]
pub struct RtcTime {
    pub year: u32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl Debug for RtcTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}-{}-{} {}:{}:{}",
            self.year, self.month, self.day, self.hour, self.minute, self.second
        )
    }
}

impl ToString for RtcTime {
    fn to_string(&self) -> String {
        format!(
            "{}:{}:{}\n{}-{}-{}",
            self.hour, self.minute, self.second, self.year, self.month, self.day
        )
    }
}

pub static RTC_DEVICE: Once<Arc<dyn RtcDevice>> = Once::new();

pub fn get_rtc_time() -> Option<RtcTime> {
    RTC_DEVICE.get().map(|rtc| rtc.read_time())
}

pub fn init_rtc(rtc: Arc<dyn RtcDevice>) {
    RTC_DEVICE.call_once(|| rtc);
}

pub trait RtcDevice: Send + Sync + DeviceBase {
    fn read_time(&self) -> RtcTime;
}
