use core::ptr::NonNull;

use fdt::node::FdtNode;
use fdt::standard_nodes::Compatible;
use fdt::Fdt;
use spin::Once;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
use virtio_drivers::transport::{DeviceType as VirtDeviceType, Transport};

use crate::board::common::get_device_info;
use crate::board::BOARD_DEVICES;
use crate::device::DeviceInfo;
use crate::device::DeviceType;

pub static DTB: Once<Fdt> = Once::new();

pub fn init_dtb(dtb: Option<usize>) {
    let fdt = if dtb.is_some() {
        unsafe { Fdt::from_ptr(dtb.unwrap() as *const u8).unwrap() }
    } else {
        panic!("No dtb found")
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
    find_virtio_device(DTB.get().unwrap());
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

/// Get the base address and irq number of the uart device from the device tree.
pub fn probe_uart() -> Option<DeviceInfo> {
    let fdt = DTB.get().unwrap();
    if let Some((base_addr, irq)) = get_device_info(fdt, "uart") {
        return Some(DeviceInfo::new(base_addr, irq));
    }
    None
}

fn find_virtio_device(fdt: &Fdt) -> Option<(usize, usize)> {
    for node in fdt.all_nodes() {
        if node.name.starts_with("virtio_mmio") {
            if let Some((device_type, info)) = virtio_probe(&node) {
                BOARD_DEVICES.lock().insert(device_type, info);
            }
        }
    }
    None
}

// keyboard
const VIRTIO5: usize = 0x10005000;
// mouse
const VIRTIO6: usize = 0x10006000;

fn virtio_probe(node: &FdtNode) -> Option<(DeviceType, DeviceInfo)> {
    if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
        let paddr = reg.starting_address as usize;
        let size = reg.size.unwrap();
        let vaddr = paddr;
        info!("walk dt addr={:#x}, size={:#x}", paddr, size);
        info!(
            "Device tree node {}: {:?}",
            node.name,
            node.compatible().map(Compatible::first),
        );
        let irq = if let Some(mut interrupts) = node.interrupts() {
            let irq = interrupts.next().unwrap();
            irq
        } else {
            0
        };

        let header = NonNull::new(vaddr as *mut VirtIOHeader).unwrap();
        match unsafe { MmioTransport::new(header) } {
            Err(_) => {}
            Ok(mut transport) => {
                info!(
                    "Detected virtio MMIO device with vendor id {:#X}, device type {:?}, version {:?}, features:{:?}",
                    transport.vendor_id(),
                    transport.device_type(),
                    transport.version(),
                    transport.read_device_features(),
                );
                info!("Probe virtio device: {:?}", transport.device_type());
                match transport.device_type() {
                    VirtDeviceType::Input => {
                        let device_info = DeviceInfo::new(paddr, irq);
                        let res = if paddr == VIRTIO5 {
                            Some((DeviceType::KeyBoardInput, device_info))
                        } else if paddr == VIRTIO6 {
                            Some((DeviceType::MouseInput, device_info))
                        } else {
                            None
                        };
                        return res;
                    }
                    VirtDeviceType::Block => {
                        let device_info = DeviceInfo::new(paddr, irq);
                        return Some((DeviceType::Block, device_info));
                    }
                    VirtDeviceType::GPU => {
                        let device_info = DeviceInfo::new(paddr, irq);
                        return Some((DeviceType::GPU, device_info));
                    }
                    _ => return None,
                }
            }
        }
    }
    None
}
