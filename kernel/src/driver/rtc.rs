use alloc::sync::Arc;

use rtc::{Rtc, RtcTime};
use spin::once::Once;

use crate::driver::DeviceBase;

static RTC: Once<Arc<Rtc>> = Once::new();

pub fn init_rtc(base_addr: usize, irq: u32) -> Arc<dyn DeviceBase> {
    info!("Init rtc, base_addr:{:#x},irq:{}", base_addr, irq);
    let rtc = Rtc::new(base_addr, irq);
    let rtc = Arc::new(rtc);
    RTC.call_once(|| rtc.clone());
    let current_time = rtc.read_time();
    println!("init rtc success, current time: {:?}", current_time);
    rtc
}

pub fn get_rtc_time() -> Option<RtcTime> {
    RTC.get().map(|rtc| rtc.read_time())
}

impl DeviceBase for Rtc {
    fn hand_irq(&self) {
        println!("rtc irq");
    }
}
