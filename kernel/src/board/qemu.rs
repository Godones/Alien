use core::ptr::NonNull;

use fdt::Fdt;
use fdt::node::FdtNode;
use fdt::standard_nodes::Compatible;
use spin::Once;
use virtio_drivers::transport::{DeviceType, Transport};
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};

use crate::board::common::{get_device_info, get_device_info_from_node};

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
    get_device_info(fdt, "uart")
}

pub fn get_gpu_info() -> Option<(usize, usize)> {
    let fdt = DTB.get().unwrap();
    find_virtio_device(&fdt, DeviceType::GPU, None)
}

pub fn get_keyboard_info() -> Option<(usize, usize)> {
    let fdt = DTB.get().unwrap();
    find_virtio_device(&fdt, DeviceType::Input, Some(VIRTIO5))
}

pub fn get_mouse_info() -> Option<(usize, usize)> {
    let fdt = DTB.get().unwrap();
    find_virtio_device(&fdt, DeviceType::Input, Some(VIRTIO6))
}

pub fn get_block_device_info() -> Option<(usize, usize)> {
    let fdt = DTB.get().unwrap();
    find_virtio_device(&fdt, DeviceType::Block, None)
}


pub fn get_net_device_info() -> Option<(usize, usize)> {
    let fdt = DTB.get().unwrap();
    find_virtio_device(&fdt, DeviceType::Network, None)
}


fn find_virtio_device(
    fdt: &Fdt,
    device_type: DeviceType,
    special_addr: Option<usize>,
) -> Option<(usize, usize)> {
    for node in fdt.all_nodes() {
        if node.name.starts_with("virtio_mmio") {
            if virtio_probe(&node, device_type, special_addr) {
                return get_device_info_from_node(&node);
            }
        }
    }
    None
}

// keyboard
const VIRTIO5: usize = 0x10005000;
// mouse
const VIRTIO6: usize = 0x10006000;

fn virtio_probe(node: &FdtNode, device_type: DeviceType, special_addr: Option<usize>) -> bool {
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
        let header = NonNull::new(vaddr as *mut VirtIOHeader).unwrap();
        match unsafe { MmioTransport::new(header) } {
            Err(_) => false,
            Ok(mut transport) => {
                info!(
                    "Detected virtio MMIO device with vendor id {:#X}, device type {:?}, version {:?}, features:{:?}",
                    transport.vendor_id(),
                    transport.device_type(),
                    transport.version(),
                    transport.read_device_features(),
                );
                info!(
                    "Probe virtio device: {:?}, special addr: {:?}",
                    transport.device_type(),
                    special_addr
                );
                if device_type == transport.device_type() {
                    if special_addr.is_none() {
                        return true;
                    } else if special_addr.is_some() && special_addr.unwrap() == paddr {
                        return true;
                    }
                }
                false
            }
        }
    } else {
        false
    }
}
