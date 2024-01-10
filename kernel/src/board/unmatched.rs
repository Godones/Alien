use crate::board::probe::Probe;
use crate::board::BOARD_DEVICES;
use crate::device::DeviceType;
use fdt::Fdt;
use spin::Once;

#[repr(align(4))]
struct Wrapper<T>(T);

pub const FDT: &[u8] = &Wrapper(*include_bytes!("../../../tools/hifive-unmatched-a00.dtb")).0;

pub static DTB: Once<Fdt> = Once::new();

pub fn init_dtb(dtb: Option<usize>) {
    let fdt = if dtb.is_some() {
        assert_eq!(dtb.unwrap(), FDT.as_ptr() as usize);
        unsafe { Fdt::from_ptr(dtb.unwrap() as *const u8).unwrap() }
    } else {
        unsafe { Fdt::from_ptr(FDT.as_ptr()).unwrap() }
    };
    DTB.call_once(|| fdt);
}

/// Get the base address and irq number of the uart device from the device tree.
pub fn probe_devices_from_dtb() {
    let dtb = crate::board::DTB.get().unwrap();
    if let Some(rtc) = dtb.probe_rtc() {
        BOARD_DEVICES.lock().insert(DeviceType::Rtc, rtc);
    }
    if let Some(uart) = dtb.probe_uart() {
        BOARD_DEVICES.lock().insert(DeviceType::Uart, uart);
    }
}
