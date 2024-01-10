use crate::board::probe::{DeviceInfo, Probe};
use crate::board::BOARD_DEVICES;
use crate::device::DeviceType;
use core::ptr::NonNull;
use fdt::Fdt;
use spin::Once;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
use virtio_drivers::transport::Transport;

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
    let dtb = DTB.get().unwrap();
    if let Some(rtc) = dtb.probe_rtc() {
        BOARD_DEVICES.lock().insert(DeviceType::Rtc, rtc);
    }
    if let Some(uart) = dtb.probe_uart() {
        BOARD_DEVICES.lock().insert(DeviceType::Uart, uart);
    }
    if let Some(virtio_mmio) = dtb.probe_virtio() {
        for virtio in virtio_mmio {
            match virtio_mmio_type(&virtio) {
                Some(ty) => {
                    BOARD_DEVICES.lock().insert(ty, virtio);
                }
                None => {
                    error!("Unknown virtio device type at {:#x}", virtio.base_addr);
                }
            }
        }
    }
}

// keyboard
const VIRTIO5: usize = 0x10005000;
// mouse
const VIRTIO6: usize = 0x10006000;

fn virtio_mmio_type(device_info: &DeviceInfo) -> Option<DeviceType> {
    let paddr = device_info.base_addr;
    let header = NonNull::new(paddr as *mut VirtIOHeader).unwrap();
    info!("walk dt addr={:#x}", paddr);
    info!(
        "Device tree node {}: {:?}",
        device_info.name, device_info.compatible
    );
    match unsafe { MmioTransport::new(header) } {
        Err(_) => None,
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
                virtio_drivers::transport::DeviceType::Input => {
                    if paddr == VIRTIO5 {
                        Some(DeviceType::KeyBoardInput)
                    } else if paddr == VIRTIO6 {
                        Some(DeviceType::MouseInput)
                    } else {
                        None
                    }
                }
                virtio_drivers::transport::DeviceType::Block => Some(DeviceType::Block),
                virtio_drivers::transport::DeviceType::GPU => Some(DeviceType::GPU),
                virtio_drivers::transport::DeviceType::Network => Some(DeviceType::Network),
                _ => None,
            }
        }
    }
}
