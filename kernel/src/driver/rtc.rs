pub use rtc::goldfish::GoldFishRtc;
use rtc::{LowRtcDevice, LowRtcDeviceExt};

use crate::device::RtcDevice;
use crate::interrupt::DeviceBase;

impl DeviceBase for GoldFishRtc {
    fn hand_irq(&self) {
        let alarm = self.read_alarm_fmt();
        let time = self.read_time_fmt();
        println!("rtc interrupt, time: {:?}, alarm: {:?}", time, alarm);
        self.clear_irq();
        self.set_alarm(self.read_time() + 1_000_000_000 * 5);
    }
}

impl RtcDevice for GoldFishRtc {}
