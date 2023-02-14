use crate::driver::hal::HalImpl;
use crate::driver::{QemuBlockDevice, QEMU_BLOCK_DEVICE};
use alloc::sync::Arc;
use core::ptr::NonNull;
use fdt::node::FdtNode;
use fdt::standard_nodes::Compatible;
use fdt::Fdt;
use virtio_drivers::device::blk::VirtIOBlk;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
use virtio_drivers::transport::{DeviceType, Transport};

pub fn init_dt(dtb: usize) {
    info!("device tree @ {:#x}", dtb);
    // Safe because the pointer is a valid pointer to unaliased memory.
    let fdt = unsafe { Fdt::from_ptr(dtb as *const u8).unwrap() };
    walk_dt(fdt);
}

fn walk_dt(fdt: Fdt) {
    for node in fdt.all_nodes() {
        if let Some(compatible) = node.compatible() {
            if compatible.all().any(|s| s == "virtio,mmio") {
                virtio_probe(node);
            }
        } else {
            // info!("{:?}",node);
        }
    }
}
fn virtio_probe(node: FdtNode) {
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
            Err(e) => warn!("Error creating VirtIO MMIO transport: {}", e),
            Ok(transport) => {
                info!(
                    "Detected virtio MMIO device with vendor id {:#X}, device type {:?}, version {:?}",
                    transport.vendor_id(),
                    transport.device_type(),
                    transport.version(),
                );
                virtio_device(transport);
            }
        }
    }
}
fn virtio_device(transport: impl Transport + 'static) {
    match transport.device_type() {
        DeviceType::Block => virtio_blk(transport),
        t => warn!("Unrecognized virtio device: {:?}", t),
    }
}
fn virtio_blk<T: Transport + 'static>(transport: T) {
    let blk = VirtIOBlk::<HalImpl, T>::new(transport).expect("failed to create blk driver");
    let qemu_block_device = QemuBlockDevice::new(blk);
    *QEMU_BLOCK_DEVICE.lock() = Some(Arc::new(qemu_block_device));
    info!("virtio-blk init finished");
}
