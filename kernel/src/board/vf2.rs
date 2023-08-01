use core::ops::Range;

use fdt::Fdt;
use spin::Once;

#[repr(align(4))]
struct _Wrapper<T>(T);

pub const FDT: &[u8] = &_Wrapper(*include_bytes!("../../../tools/jh7110-visionfive-v2.dtb")).0;

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
///
/// Return:
/// (base addr, irq)
pub fn get_rtc_info() -> Option<(usize, usize)> {
    let fdt = DTB.get().unwrap();
    // get_device_info(fdt, "rtc")
    let node = fdt.all_nodes().find(|node| node.name.starts_with("rtc"));
    assert!(node.is_some());
    let node = node.unwrap();
    let reg = node.reg().unwrap().next().unwrap();
    let range = Range {
        start: reg.starting_address as usize,
        end: reg.starting_address as usize + reg.size.unwrap(),
    };
    // let irq = node.property("interrupts").unwrap().value;
    // let irq = u32::from_be_bytes(irq.try_into().unwrap());
    let irq = 0xa;
    Some((range.start, irq as usize))
}

/// Get the base address and irq number of the uart device from the device tree.
///
/// Return:
/// (base addr, irq)
pub fn get_uart_info() -> Option<(usize, usize)> {
    let fdt = DTB.get().unwrap();
    // get_device_info(fdt, "serial")
    let node = fdt.all_nodes().find(|node| node.name.starts_with("serial"));
    assert!(node.is_some());
    let node = node.unwrap();
    let mut reg = node.reg().unwrap();
    let irq = node.property("interrupts").unwrap().value;
    let irq = u32::from_be_bytes(irq.try_into().unwrap());
    let base_addr = reg.next().unwrap().starting_address as usize;
    Some((base_addr, irq as usize))
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
