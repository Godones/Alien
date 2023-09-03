use alloc::sync::Arc;
use rtc::RtcTime;

use spin::Once;

use crate::interrupt::DeviceBase;

pub static RTC_DEVICE: Once<Arc<dyn RtcDevice>> = Once::new();

pub fn get_rtc_time() -> Option<RtcTime> {
    RTC_DEVICE.get().map(|rtc| rtc.read_time_fmt())
}

pub fn init_rtc(rtc: Arc<dyn RtcDevice>) {
    RTC_DEVICE.call_once(|| rtc);
}

pub trait RtcDevice: Send + Sync + DeviceBase + rtc::LowRtcDevice + rtc::LowRtcDeviceExt {}

#[allow(dead_code)]
fn example() {
    // let rtc = RTC_DEVICE.get().unwrap();
    // let time = rtc.read_time();
    // let alarm = rtc.read_alarm();
    // println!("time: {:#x}, alarm: {:#x}", time, alarm);
    // rtc.clear_irq();
    // rtc.enable_irq();
    // println!("set alarm");
    // rtc.set_alarm(time + 1_000_000_000 * 5); // wait 5s
    // let alarm = rtc.read_alarm_fmt();
    // let status = rtc.alarm_status();
    // let is_enable = rtc.is_irq_enabled();
    // println!(
    //     "At {:?}, rtc will interrupt, status: {} enable: {}",
    //     alarm, status, is_enable
    // );
    // loop {
    //     let time = rtc.read_time();
    //     let alarm = rtc.read_alarm();
    //     if time == alarm {
    //         let status = rtc.alarm_status();
    //         let enable = rtc.is_irq_enabled();
    //         println!("time == alarm, status: {}, enable: {}", status, enable);
    //     }
    // }
}
