#![no_std]

mod block;
mod gpu;
mod input;
mod net;
mod prob;
mod rtc;
mod uart;

extern crate alloc;

use alloc::{boxed::Box, sync::Arc, vec::Vec};
use core::ptr::NonNull;

pub use block::{BLKDevice, BLOCK_DEVICE};
use config::MAX_INPUT_EVENT_NUM;
use device_interface::{DeviceBase, GpuDevice, LowBlockDevice};
use drivers::{
    block_device::GenericBlockDevice,
    rtc::GoldFishRtc,
    uart::{Uart, Uart16550, Uart8250},
};
use fdt::Fdt;
pub use gpu::{GPUDevice, GPU_DEVICE};
pub use input::{INPUTDevice, KEYBOARD_INPUT_DEVICE, MOUSE_INPUT_DEVICE};
use interrupt::register_device_to_plic;
use log::info;
use platform::println;
pub use rtc::{RTCDevice, RTC_DEVICE};
pub use uart::{UARTDevice, UART_DEVICE};
use virtio_drivers::transport::{
    mmio::{MmioTransport, VirtIOHeader},
    DeviceType, Transport,
};

use crate::prob::Probe;

pub struct DeviceInfo {
    pub device: Arc<dyn DeviceBase>,
    pub irq: usize,
    pub need_register: bool,
}

/// Probe all devices from device tree and init them.
/// # Warning
/// Before init device, we should init platform first.
///
pub fn init_device() {
    let dtb_ptr = platform::platform_dtb_ptr();

    let dtb = unsafe { Fdt::from_ptr(dtb_ptr as *const u8).unwrap() };
    match dtb.probe_rtc() {
        Some(rtc) => init_rtc(rtc),
        None => {
            println!("There is no rtc device");
        }
    }

    match dtb.probe_uart() {
        Some(uart) => init_uart(uart),
        None => {
            println!("There is no uart device");
        }
    }

    #[cfg(not(all(feature = "vf2", feature = "hifive")))]
    {
        match dtb.probe_virtio() {
            Some(virtio_mmio_devices) => init_virtio_mmio(virtio_mmio_devices),
            None => {
                println!("There is no virtio-mmio device");
            }
        }
    }

    #[cfg(feature = "vf2")]
    match dtb.probe_sdio() {
        Some(sdio) => init_block_device(sdio, None),
        None => {
            panic!("There is no sdio device");
        }
    }

    #[cfg(feature = "hifive")]
    init_block_device(
        prob::DeviceInfo::new(
            alloc::string::String::new(),
            0,
            0,
            alloc::string::String::new(),
        ),
        None,
    );

    #[cfg(any(feature = "hifive", feature = "vf2"))]
    init_net(None);
}

fn init_rtc(rtc: prob::DeviceInfo) {
    let info = rtc;
    println!(
        "Init rtc, base_addr:{:#x}, irq:{}",
        info.base_addr, info.irq
    );
    match info.compatible.as_str() {
        "google,goldfish-rtc" => {
            let rtc = Arc::new(GoldFishRtc::new(info.base_addr));
            let current_time = rtc.read_time_string();
            rtc::init_rtc(rtc.clone());
            register_device_to_plic(info.irq, rtc);
            println!("Init rtc success, current time: {:?}", current_time);
        }
        name => {
            println!("Don't support rtc: {}", name);
        }
    }
}

fn init_uart(uart: prob::DeviceInfo) {
    let (base_addr, irq) = (uart.base_addr, uart.irq);
    println!("Init uart, base_addr:{:#x},irq:{}", base_addr, irq);
    match uart.compatible.as_str() {
        "ns16550a" => {
            // qemu
            let uart = Uart16550::new(base_addr);
            let uart = Arc::new(Uart::new(Box::new(uart)));
            uart::init_uart(uart.clone());
            register_device_to_plic(irq, uart);
        }
        "snps,dw-apb-uart" => {
            // vf2
            let uart = Uart8250::new(base_addr);
            let uart = Arc::new(Uart::new(Box::new(uart)));
            uart::init_uart(uart.clone());
            register_device_to_plic(irq, uart);
        }
        name => {
            panic!("Don't support uart: {}", name);
        }
    }
    println!("Init uart success");
}

// keyboard
const VIRTIO5: usize = 0x10005000;
// mouse
const VIRTIO6: usize = 0x10006000;

pub fn init_virtio_mmio(devices: Vec<prob::DeviceInfo>) {
    for device in devices {
        let paddr = device.base_addr;
        let header = NonNull::new(paddr as *mut VirtIOHeader).unwrap();
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
                    DeviceType::Input => {
                        if paddr == VIRTIO5 {
                            init_input_device(device, "keyboard", Some(transport));
                        } else if paddr == VIRTIO6 {
                            init_input_device(device, "mouse", Some(transport));
                        }
                    }
                    DeviceType::Block => init_block_device(device, Some(transport)),
                    DeviceType::GPU => init_gpu(device, Some(transport)),
                    DeviceType::Network => init_net(Some(device)),
                    ty => {
                        println!("Don't support virtio device type: {:?}", ty);
                    }
                }
            }
        }
    }
}

#[cfg(feature = "ramdisk")]
static RAMDISK: &'static [u8] = include_bytes!("../../../tools/sdcard.img");

#[cfg(feature = "ramdisk")]
pub fn checkout_fs_img() {
    let img_start = RAMDISK.as_ptr() as usize;
    let img_size = RAMDISK.len();
    println!("img_start: {:#x}, img_size: {:#x}", img_start, img_size);
}

fn init_block_device(blk: prob::DeviceInfo, mmio_transport: Option<MmioTransport>) {
    use drivers::block_device::VirtIOBlkWrapper;
    let (base_addr, irq) = (blk.base_addr, blk.irq);
    match blk.compatible.as_str() {
        "virtio,mmio" => {
            // qemu
            // let mut block_device = VirtIOBlkWrapper::new(blk.base_addr);
            let block_device = VirtIOBlkWrapper::from_mmio(mmio_transport.unwrap());
            println!("Init block device, base_addr:{:#x},irq:{}", base_addr, irq);
            let size = block_device.capacity();
            println!("Block device size is {}MB", size * 512 / 1024 / 1024);
            let block_device = Arc::new(GenericBlockDevice::new(Box::new(block_device)));
            block::init_block_device(block_device);
            // register_device_to_plic(irq, block_device);
            println!("Init block device success");
        }
        "starfive,jh7110-sdio" => {
            // starfive2
            #[cfg(not(feature = "ramdisk"))]
            {
                use arch::read_timer;
                use platform::config::CLOCK_FREQ;
                pub fn sleep(ms: usize) {
                    let start = read_timer();
                    while read_timer() - start < ms * (CLOCK_FREQ / 1000) {
                        core::hint::spin_loop();
                    }
                }
                use drivers::block_device::{VF2SDDriver, Vf2SdDriver};
                let block_device = VF2SDDriver::new(Vf2SdDriver::new(sleep));
                let size = block_device.capacity();
                println!("Block device size is {}MB", size * 512 / 1024 / 1024);
                let block_device = Arc::new(GenericBlockDevice::new(Box::new(block_device)));
                block::init_block_device(block_device);
                // register_device_to_plic(irq, block_device);
                println!("Init SDIO block device success");
            }
            #[cfg(feature = "ramdisk")]
            {
                init_ramdisk();
            }
        }
        name => {
            println!("Don't support block device: {}", name);
            #[cfg(feature = "ramdisk")]
            {
                init_ramdisk();
            }
            #[cfg(not(feature = "ramdisk"))]
            panic!("System need block device, but there is no block device");
        }
    }
}

#[cfg(feature = "ramdisk")]
fn init_ramdisk() {
    use drivers::block_device::MemoryFat32Img;
    checkout_fs_img();
    let data =
        unsafe { core::slice::from_raw_parts_mut(RAMDISK.as_ptr() as *mut u8, RAMDISK.len()) };
    let block_device = GenericBlockDevice::new(Box::new(MemoryFat32Img::new(data)));
    let block_device = Arc::new(block_device);
    block::init_block_device(block_device);
    println!("Init fake block device success");
}

fn init_gpu(gpu: prob::DeviceInfo, mmio_transport: Option<MmioTransport>) {
    let (base_addr, irq) = (gpu.base_addr, gpu.irq);
    println!("Init gpu, base_addr:{:#x},irq:{}", base_addr, irq);
    match gpu.compatible.as_str() {
        "virtio,mmio" => {
            // qemu
            use drivers::gpu::VirtIOGpuWrapper;
            let gpu = VirtIOGpuWrapper::from_mmio(mmio_transport.unwrap());
            let resolution = gpu.resolution();
            println!("GPU resolution: {:?}", resolution);
            let gpu = Arc::new(gpu);
            gpu::init_gpu(gpu);
            // let _ = register_device_to_plic(irq, gpu);
            println!("Init gpu success");
        }
        name => {
            println!("Don't support gpu: {}", name);
        }
    }
}

fn init_input_device(input: prob::DeviceInfo, name: &str, mmio_transport: Option<MmioTransport>) {
    let (base_addr, irq) = (input.base_addr, input.irq);
    println!(
        "Init {} input device, base_addr:{:#x},irq:{}",
        name, base_addr, irq
    );
    match input.compatible.as_str() {
        "virtio,mmio" => {
            // qemu
            use drivers::input::VirtIOInputDriver;
            let input =
                VirtIOInputDriver::from_mmio(mmio_transport.unwrap(), MAX_INPUT_EVENT_NUM as u32);
            let input = Arc::new(input);
            match name {
                "mouse" => input::init_mouse_input_device(input.clone()),
                "keyboard" => input::init_keyboard_input_device(input.clone()),
                _ => panic!("Don't support {} input device", name),
            }
            let _ = register_device_to_plic(irq, input);
            println!("Init keyboard input device success");
        }
        name => {
            println!("Don't support keyboard input device: {}", name);
        }
    }
}

fn init_net(_nic: Option<prob::DeviceInfo>) {
    // If we need run test, we should init loop device because no we can't route packet
    #[cfg(feature = "test")]
    {
        init_loop_device();
    }
    #[cfg(not(feature = "test"))]
    {
        let nic = _nic.unwrap();
        let (base_addr, irq) = (nic.base_addr, nic.irq);
        println!("Init net device, base_addr:{:#x},irq:{}", base_addr, irq);

        match nic.compatible.as_str() {
            "virtio,mmio" => {
                use core::str::FromStr;

                use config::{QEMU_GATEWAY, QEMU_IP};
                use drivers::net::{NetNeedFunc, VirtIONetDriver};
                use smoltcp::wire::IpAddress;
                let virtio_net = VirtIONetDriver::from_mmio(base_addr);
                let device = Box::new(virtio_net);
                netcore::init_net(
                    device,
                    Arc::new(NetNeedFunc),
                    IpAddress::from_str(QEMU_IP).unwrap(),
                    IpAddress::from_str(QEMU_GATEWAY).unwrap(),
                    true,
                );
                println!("Init net device success");
            }
            name => {
                panic!("Don't support net device: {}", name);
            }
        }
    }
}

#[cfg(feature = "test")]
fn init_loop_device() {
    use drivers::net::{LoopbackDev, NetNeedFunc};
    use smoltcp::wire::IpAddress;
    // use default ip and gateway for qemu
    let ip = IpAddress::v4(127, 0, 0, 1);
    let gate_way = IpAddress::v4(127, 0, 0, 1);
    let loopback = Box::new(LoopbackDev::new());
    netcore::init_net(loopback, Arc::new(NetNeedFunc), ip, gate_way, false);
    println!("Init net device success");
}
