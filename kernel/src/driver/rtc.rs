use rtc::{Rtc, RtcTime};
use spin::once::Once;

static RTC: Once<Rtc> = Once::new();

pub fn init_rtc(base_addr: usize, irq: u32) {
    info!("Init rtc, base_addr:{:#x},irq:{}", base_addr, irq);
    let rtc = Rtc::new(base_addr, irq);
    RTC.call_once(|| rtc);
}

pub fn get_rtc_time() -> Option<RtcTime> {
    RTC.get().map(|rtc| rtc.read_time())
}
