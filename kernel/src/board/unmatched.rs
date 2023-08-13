use alloc::vec::Vec;
use fdt::Fdt;
use spin::Once;

use crate::board::common::get_device_info;

#[repr(align(4))]
struct _Wrapper<T>(T);

pub const FDT: &[u8] = &_Wrapper(*include_bytes!("../../../tools/hifive-unmatched-a00.dtb")).0;

pub static DTB: Once<Fdt> = Once::new();

pub fn init_dtb(dtb: Option<usize>) {
    let fdt = if dtb.is_some() {
        unsafe { Fdt::from_ptr(dtb as *const u8).unwrap() }
    } else {
        unsafe { Fdt::from_ptr(FDT.as_ptr()).unwrap() }
    };
    DTB.call_once(|| fdt);
}

/// Get the base address and irq number of the uart device from the device tree.
pub fn probe_devices_from_dtb() {
    if let Some(rtc) = probe_rtc() {
        BOARD_DEVICES.lock().insert(DeviceType::Rtc, rtc);
    }
    if let Some(uart) = probe_uart() {
        BOARD_DEVICES.lock().insert(DeviceType::Uart, uart);
    }
}

/// Get the base address and irq number of the uart device from the device tree.
pub fn probe_rtc() -> Option<DeviceInfo> {
    let fdt = DTB.get().unwrap();
    let res = get_device_info(fdt, "rtc");
    if res.is_none() {
        return None;
    }
    let (base_addr, irq) = res.unwrap();
    Some(DeviceInfo::new(base_addr, irq))
}

pub fn probe_uart() -> Option<DeviceInfo> {
    let fdt = DTB.get().unwrap();
    if let Some((base_addr, irq)) = get_device_info(fdt, "serial") {
        return Some(DeviceInfo::new(base_addr, irq));
    }
    None
}
