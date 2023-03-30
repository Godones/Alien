use crate::arch::hart_id;
use crate::config::CPU_NUM;
use crate::driver::hal::HalImpl;
use crate::driver::rtc::init_rtc;
use crate::driver::uart::init_uart;
use crate::driver::DeviceBase;
use crate::driver::{pci_probe, QemuBlockDevice, QEMU_BLOCK_DEVICE};
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::ptr::NonNull;
use fdt::node::FdtNode;
use fdt::standard_nodes::Compatible;
use fdt::Fdt;
use lazy_static::lazy_static;
use plic::{Mode, PLIC};
use spin::{Mutex, Once};
use virtio_drivers::device::blk::VirtIOBlk;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
use virtio_drivers::transport::{DeviceType, Transport};

pub static PLIC: Once<PLIC> = Once::new();

lazy_static! {
    pub static ref DEVICE_TABLE: Mutex<BTreeMap<usize, Arc<dyn DeviceBase>>> =
        Mutex::new(BTreeMap::new());
}

pub fn init_dt(dtb: usize) {
    println!("device tree @ {:#x}", dtb);
    // Safe because the pointer is a valid pointer to unaliased memory.
    let fdt = unsafe { Fdt::from_ptr(dtb as *const u8).unwrap() };
    init_plic(&fdt);
    walk_dt(&fdt);
}

fn init_plic(fdt: &Fdt) {
    for node in fdt.all_nodes() {
        if node.name.starts_with("plic") {
            // let cpus  = fdt.cpus().count();
            let addr = node.reg().unwrap().next().unwrap().starting_address as usize;
            let privileges = [2; CPU_NUM];
            PLIC.call_once(|| PLIC::new(addr, &privileges));
        }
    }
}

fn walk_dt(fdt: &Fdt) {
    for node in fdt.all_nodes() {
        if node.name.starts_with("virtio_mmio") {
            println!("probe virtio_mmio device");
            // init_device_to_plic(node);
            virtio_probe(node);
        } else if node.name.starts_with("pci") {
            pci_probe(node);
        } else if node.name.starts_with("rtc") {
            println!("probe rtc device");
            rtc_probe(node);
        } else if node.name.starts_with("uart") {
            println!("probe uart device");
            uart_probe(node)
        }
    }
}

fn uart_probe(node: FdtNode) {
    if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
        let paddr = reg.starting_address as usize;
        let irq = init_device_to_plic(node);
        if irq != 0 {
            let uart = init_uart(paddr);
            let mut table = DEVICE_TABLE.lock();
            table.insert(irq, uart);
        }
    }
}

fn init_device_to_plic(node: FdtNode) -> usize {
    if let Some(interrupts) = node.interrupts() {
        let vec = interrupts.map(|x| x).collect::<Vec<usize>>();
        if vec.len() > 0 {
            let irq = vec[0] as u32;
            let plic = PLIC.get().unwrap();
            let hard_id = hart_id();
            println!(
                "plic enable irq {} for hart {}, priority {}",
                irq, hard_id, 1
            );
            plic.set_threshold(hard_id as u32, Mode::Machine, 1);
            plic.set_threshold(hard_id as u32, Mode::Supervisor, 0);
            plic.enable(hard_id as u32, Mode::Supervisor, irq);
            plic.set_priority(irq, 1);
            return irq as usize;
        }
    }
    0
}

fn rtc_probe(node: FdtNode) {
    if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
        let paddr = reg.starting_address as usize;
        let irq = init_device_to_plic(node);
        if irq != 0 {
            let rtc = init_rtc(paddr, irq as u32);
            let mut table = DEVICE_TABLE.lock();
            table.insert(irq, rtc);
        }
    };
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
fn virtio_device(transport: MmioTransport) {
    match transport.device_type() {
        DeviceType::Block => virtio_blk(transport),
        t => warn!("Unrecognized virtio device: {:?}", t),
    }
}
fn virtio_blk(transport: MmioTransport) {
    let blk =
        VirtIOBlk::<HalImpl, MmioTransport>::new(transport).expect("failed to create blk driver");
    let qemu_block_device = QemuBlockDevice::new(blk);
    QEMU_BLOCK_DEVICE.lock().push(Arc::new(qemu_block_device));
    info!("virtio-blk init finished");
}
