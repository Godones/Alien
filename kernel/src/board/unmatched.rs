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
///
/// Return:
/// (base addr, irq)
pub fn get_rtc_info() -> Option<(usize, usize)> {
    let fdt = DTB.get().unwrap();
    get_device_info(fdt, "rtc")
}

/// Get the base address and irq number of the uart device from the device tree.
///
/// Return:
/// (base addr, irq)
pub fn get_uart_info() -> Option<(usize, usize)> {
    let fdt = DTB.get().unwrap();
    get_device_info(fdt, "serial0")
}

pub fn get_gpu_info() -> Option<(usize, usize)> {
    None
}

pub fn get_keyboard_info() -> Option<(usize, usize)> {
    None
}

pub fn get_mouse_info() -> Option<(usize, usize)> {
    None
}

pub fn get_block_device_info() -> Option<(usize, usize)> {
    None
}

pub fn get_net_device_info() -> Option<(usize, usize)> {
    None
}