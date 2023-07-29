use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::ptr::NonNull;

use fdt::Fdt;
use fdt::node::FdtNode;
use fdt::standard_nodes::Compatible;
use hashbrown::HashMap;
use lazy_static::lazy_static;
use spin::Once;
use virtio_drivers::device::blk::VirtIOBlk;
use virtio_drivers::device::gpu::VirtIOGpu;
use virtio_drivers::device::input::VirtIOInput;
use virtio_drivers::transport::{DeviceType, Transport};
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};

use kernel_sync::Mutex;
use plic::{Mode, PLIC};

use crate::arch::hart_id;
use crate::config::{CPU_NUM, MAX_INPUT_EVENT_NUM};
use crate::driver::{GenericBlockDevice, pci_probe, QEMU_BLOCK_DEVICE};
use crate::driver::DeviceBase;
use crate::driver::gpu::{GPU_DEVICE, VirtIOGpuWrapper};
use crate::driver::hal::HalImpl;
use crate::driver::input::{INPUT_DEVICE, InputDriver};
use crate::driver::rtc::init_rtc;
use crate::driver::uart::init_uart;

pub static PLIC: Once<PLIC> = Once::new();

lazy_static! {
    pub static ref DEVICE_TABLE: Mutex<BTreeMap<usize, Arc<dyn DeviceBase>>> =
        Mutex::new(BTreeMap::new());
}

pub fn init_dt(dtb: usize) {
    println!("device tree @{:#x}", dtb);
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
            // println!("probe virtio_mmio device");
            // init_device_to_plic(node);
            virtio_probe(node);
        } else if node.name.starts_with("pci") {
            pci_probe(node);
        } else if node.name.starts_with("rtc") {
            println!("probe rtc device");
            rtc_probe(node);
        }

        #[cfg(not(any(feature = "vf2", feature = "cv1811h")))]
        if node.name.starts_with("uart") {
            println!("probe uart device");
            uart_probe(node)
        }

        #[cfg(feature = "vf2")]
        if node.name.starts_with("uart") {
            // println!("probe uart device");
            // uart_probe(node)
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
            Ok(mut transport) => {
                info!(
                    "Detected virtio MMIO device with vendor id {:#X}, device type {:?}, version {:?}, features:{:?}",
                    transport.vendor_id(),
                    transport.device_type(),
                    transport.version(),
                    transport.read_device_features(),
                );
                let irq = match transport.device_type() {
                    DeviceType::Input => {
                        let irq = init_device_to_plic(node);
                        irq
                    }
                    _ => 0,
                };
                virtio_device(transport, paddr, irq);
            }
        }
    }
}

fn virtio_device(transport: MmioTransport, addr: usize, irq: usize) {
    match transport.device_type() {
        DeviceType::Block => virtio_blk(transport),
        DeviceType::GPU => virtio_gpu(transport),
        DeviceType::Input => virto_input(transport, addr, irq),
        t => warn!("Unrecognized virtio device: {:?}", t),
    }
}

fn virtio_blk(transport: MmioTransport) {
    let blk =
        VirtIOBlk::<HalImpl, MmioTransport>::new(transport).expect("failed to create blk driver");
    let size = blk.capacity();
    println!("blk device size is {}MB", size * 512 / 1024 / 1024);
    let qemu_block_device = GenericBlockDevice::new(Box::new(blk));
    QEMU_BLOCK_DEVICE.lock().push(Arc::new(qemu_block_device));
    println!("virtio-blk init finished");
}

fn virtio_gpu(transport: MmioTransport) {
    let gpu =
        VirtIOGpu::<HalImpl, MmioTransport>::new(transport).expect("failed to create gpu driver");
    let qemu_gpu_device = VirtIOGpuWrapper::new(gpu);
    GPU_DEVICE.call_once(|| Arc::new(qemu_gpu_device));
    println!("virtio-gpu init finished");
}

const VIRTIO5: usize = 0x10005000;
const VIRTIO6: usize = 0x10006000;

fn virto_input(transport: MmioTransport, addr: usize, irq: usize) {
    let input = VirtIOInput::<HalImpl, MmioTransport>::new(transport)
        .expect("failed to create input driver");
    let qemu_input_device = InputDriver::new(input, MAX_INPUT_EVENT_NUM as u32);
    let input_device = Arc::new(qemu_input_device);
    unsafe {
        if INPUT_DEVICE.get().is_none() {
            INPUT_DEVICE.call_once(|| HashMap::new());
        }
        if addr == VIRTIO5 {
            let map = INPUT_DEVICE.get_mut().unwrap();
            map.insert("keyboard", input_device.clone());
        } else if addr == VIRTIO6 {
            let map = INPUT_DEVICE.get_mut().unwrap();
            map.insert("mouse", input_device.clone());
        }
    }
    let mut table = DEVICE_TABLE.lock();
    table.insert(irq, input_device);
    println!("virtio-input init finished");
}
