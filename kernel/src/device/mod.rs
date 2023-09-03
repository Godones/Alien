use alloc::boxed::Box;
use alloc::sync::Arc;
use core::sync::atomic::Ordering;
use virtio_drivers::transport::mmio::MmioTransport;

pub use block::{BlockDevice, BLOCK_DEVICE};
pub use gpu::{GpuDevice, GPU_DEVICE};
pub use input::{sys_event_get, InputDevice, KEYBOARD_INPUT_DEVICE, MOUSE_INPUT_DEVICE};

use crate::board::{get_rtc_info, probe_devices_from_dtb};
use crate::driver::hal::HalImpl;
use crate::driver::uart::Uart;
use crate::driver::GenericBlockDevice;
use crate::interrupt::register_device_to_plic;
use crate::print::console::UART_FLAG;

// pub use pci::{pci_probe,pci_read,pci_write};
pub use self::rtc::{get_rtc_time, RtcDevice, RTC_DEVICE};
pub use self::uart::{UartDevice, UART_DEVICE};

mod block;
mod gpu;
mod input;
mod net;
mod pci;
mod rtc;
mod uart;

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Hash, Ord, Eq)]
pub enum DeviceType {
    Block,
    GPU,
    KeyBoardInput,
    MouseInput,
    Network,
    Uart,
    Rtc,
}

pub struct DeviceInfo {
    pub base_addr: usize,
    pub irq: usize,
}

impl DeviceInfo {
    pub fn new(base_addr: usize, irq: usize) -> Self {
        Self { base_addr, irq }
    }
}

pub fn init_device() {
    probe_devices_from_dtb();
    init_uart();
    init_gpu();
    init_keyboard_input_device();
    init_mouse_input_device();
    init_rtc();
    init_net();
    // in qemu, we can't init block device before other virtio device now
    init_block_device();
    // init_pci();
}

fn init_rtc() {
    let res = get_rtc_info();
    if res.is_none() {
        println!("There is no rtc device");
        return;
    }
    let (base_addr, irq) = res.unwrap();
    println!("Init rtc, base_addr:{:#x},irq:{}", base_addr, irq);
    let rtc: Arc<dyn RtcDevice>;
    #[cfg(feature = "qemu")]
    {
        use crate::driver::rtc::GoldFishRtc;
        rtc = Arc::new(GoldFishRtc::new(base_addr))
    }
    let current_time = rtc.read_time_fmt();
    rtc::init_rtc(rtc.clone());
    register_device_to_plic(irq, rtc.clone());
    println!("init rtc success, current time: {:?}", current_time);
}

fn init_uart() {
    let res = crate::board::get_uart_info();
    if res.is_none() {
        println!("There is no uart device");
        return;
    }
    let (base_addr, irq) = res.unwrap();
    println!("Init uart, base_addr:{:#x},irq:{}", base_addr, irq);
    #[cfg(feature = "qemu")]
    {
        use ::uart::Uart16550Raw;
        let uart = Uart16550Raw::new(base_addr);
        let uart = Uart::new(Box::new(uart));
        let uart = Arc::new(uart);
        uart::init_uart(uart.clone());
        register_device_to_plic(irq, uart);
    }
    #[cfg(feature = "vf2")]
    {
        use ::uart::Uart8250Raw;
        let uart = Uart8250Raw::<4>::new(base_addr);
        let uart = Uart::new(Box::new(uart));
        let uart = Arc::new(uart);
        uart::init_uart(uart.clone());
        register_device_to_plic(irq, uart);
    }
    UART_FLAG.store(true, Ordering::Relaxed);
    println!("init uart success");
}

fn init_gpu() {
    let res = crate::board::get_gpu_info();
    if res.is_none() {
        println!("There is no gpu device");
        return;
    }
    let (base_addr, irq) = res.unwrap();
    println!("Init gpu, base_addr:{:#x},irq:{}", base_addr, irq);
    #[cfg(feature = "qemu")]
    {
        use crate::driver::gpu::VirtIOGpuWrapper;
        let gpu = VirtIOGpuWrapper::new(base_addr);
        let gpu = Arc::new(gpu);
        gpu::init_gpu(gpu.clone());
        // let _ = register_device_to_plic(irq, gpu);
        println!("init gpu success");
    }
}

fn init_block_device() {
    #[cfg(feature = "qemu")]
    {
        use crate::driver::VirtIOBlkWrapper;
        let res = crate::board::get_block_device_info();
        if res.is_none() {
            println!("There is no block device");
            return;
        }
        let (base_addr, irq) = res.unwrap();
        println!("Init block device, base_addr:{:#x},irq:{}", base_addr, irq);
        let block_device = VirtIOBlkWrapper::new(base_addr);
        let block_device = GenericBlockDevice::new(Box::new(block_device));
        let block_device = Arc::new(block_device);
        block::init_block_device(block_device);
        // register_device_to_plic(irq, block_device);
        println!("init block device success");
    }
    #[cfg(any(feature = "vf2", feature = "hifive"))]
    {
        use crate::board::checkout_fs_img;
        checkout_fs_img();
        init_fake_disk();
        println!("init fake disk success");
    }
}

#[cfg(any(feature = "vf2", feature = "hifive"))]
fn init_fake_disk() {
    use crate::board::{img_end, img_start};
    use crate::driver::MemoryFat32Img;
    let data = unsafe {
        core::slice::from_raw_parts_mut(img_start as *mut u8, img_end as usize - img_start as usize)
    };
    let device = GenericBlockDevice::new(Box::new(MemoryFat32Img::new(data)));
    let device = Arc::new(device);
    block::init_block_device(device.clone());
}

fn init_keyboard_input_device() {
    let res = crate::board::get_keyboard_info();
    if res.is_none() {
        println!("There is no keyboard device");
        return;
    }
    let (base_addr, irq) = res.unwrap();
    println!(
        "Init keyboard input device, base_addr:{:#x},irq:{}",
        base_addr, irq
    );
    #[cfg(feature = "qemu")]
    {
        use crate::config::MAX_INPUT_EVENT_NUM;
        use crate::driver::input::VirtIOInputDriver;
        let input_device = VirtIOInputDriver::from_addr(base_addr, MAX_INPUT_EVENT_NUM as u32);
        let input_device = Arc::new(input_device);
        input::init_keyboard_input_device(input_device.clone());
        let _ = register_device_to_plic(irq, input_device);
        println!("init keyboard input device success");
    }
}

fn init_mouse_input_device() {
    let res = crate::board::get_mouse_info();
    if res.is_none() {
        println!("There is no mouse device");
        return;
    }
    let (base_addr, irq) = res.unwrap();
    println!(
        "Init mouse input device, base_addr:{:#x},irq:{}",
        base_addr, irq
    );
    #[cfg(feature = "qemu")]
    {
        use crate::config::MAX_INPUT_EVENT_NUM;
        use crate::driver::input::VirtIOInputDriver;
        let input_device = VirtIOInputDriver::from_addr(base_addr, MAX_INPUT_EVENT_NUM as u32);
        let input_device = Arc::new(input_device);
        input::init_mouse_input_device(input_device.clone());
        let _ = register_device_to_plic(irq, input_device);
        println!("init mouse input device success");
    }
}

fn init_net() {
    let res = crate::board::get_net_device_info();
    if res.is_none() {
        init_loop_device();
        println!("There is no net device");
        return;
    }
    let (base_addr, irq) = res.unwrap();
    println!("Init net device, base_addr:{:#x},irq:{}", base_addr, irq);
    #[cfg(feature = "qemu")]
    {
        use crate::device::net::NetNeedFunc;
        use crate::driver::net::VirtIONetDeviceWrapper;
        let mut net_device = VirtIONetDeviceWrapper::from_addr(base_addr);
        // let _ = register_device_to_plic(irq, net_device);
        let net_device = net_device.take().unwrap();
        // use default ip and gateway for qemu
        simple_net::init_net(
            Some(net_device),
            Arc::new(NetNeedFunc),
            None,
            None,
            false,
            true,
        );
        println!("init net device success");
        println!("test echo-server");
        #[cfg(feature = "net_test")]
        net::nettest::accept_loop();
    }
}

fn init_loop_device() {
    use crate::device::net::NetNeedFunc;
    // use default ip and gateway for qemu
    simple_net::init_net::<HalImpl, MmioTransport, 64>(
        None,
        Arc::new(NetNeedFunc),
        None,
        None,
        false,
        true,
    );
    println!("init net device success");
}
