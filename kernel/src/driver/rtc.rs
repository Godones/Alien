pub use rtc::Rtc;

use crate::device::RtcDevice;
use crate::interrupt::DeviceBase;

impl RtcDevice for Rtc {
    fn read_time(&self) -> crate::device::RtcTime {
        let time = self.read_time();
        crate::device::RtcTime {
            year: time.year,
            month: time.month,
            day: time.day,
            hour: time.hour,
            minute: time.minute,
            second: time.second,
        }
    }
}

impl DeviceBase for Rtc {
    fn hand_irq(&self) {
        println!("rtc irq");
    }
}
