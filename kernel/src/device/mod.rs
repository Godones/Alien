pub use self::rtc::{get_rtc_time, RTCDevice, RtcDevice, RTC_DEVICE};
pub use self::uart::{UARTDevice, UartDevice, UART_DEVICE};
use crate::board::{get_rtc_info, probe_devices_from_dtb};
use crate::driver::uart::Uart;
use crate::driver::GenericBlockDevice;
use crate::interrupt::register_device_to_plic;
use crate::print::console::UART_FLAG;
use alloc::boxed::Box;
use alloc::sync::Arc;
pub use block::{BLKDevice, BlockDevice, BLOCK_DEVICE};
use core::sync::atomic::Ordering;
use smoltcp::wire::IpAddress;
pub use gpu::{GPUDevice, GpuDevice, GPU_DEVICE};
pub use input::{
    sys_event_get, INPUTDevice, InputDevice, KEYBOARD_INPUT_DEVICE, MOUSE_INPUT_DEVICE,
};

mod block;
mod gpu;
mod input;
mod net;
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
    #[cfg(feature = "qemu")]
    {
        use crate::driver::rtc::GoldFishRtc;
        let rtc: Arc<dyn RtcDevice>;
        rtc = Arc::new(GoldFishRtc::new(base_addr));
        let current_time = rtc.read_time_fmt();
        rtc::init_rtc(rtc.clone());
        register_device_to_plic(irq, rtc.clone());
        println!("Init rtc success, current time: {:?}", current_time);
    }
    #[cfg(feature = "vf2")]
    {
        // rtc = todo!();
        println!("Don't support rtc on vf2");
    }
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
        use crate::driver::uart::Uart16550;
        let uart = Uart16550::new(base_addr);
        let uart = Uart::new(Box::new(uart));
        let uart = Arc::new(uart);
        uart::init_uart(uart.clone());
        register_device_to_plic(irq, uart);
    }
    #[cfg(feature = "vf2")]
    {
        use crate::driver::uart::Uart8250;
        let uart = Uart8250::new(base_addr);
        let uart = Uart::new(Box::new(uart));
        let uart = Arc::new(uart);
        uart::init_uart(uart.clone());
        register_device_to_plic(irq, uart);
    }
    UART_FLAG.store(true, Ordering::Relaxed);
    println!("Init uart success");
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
        println!("Init gpu success");
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
        println!("Init block device success");
    }
    #[cfg(any(feature = "vf2", feature = "hifive"))]
    {
        use crate::board::checkout_fs_img;
        checkout_fs_img();
        init_fake_disk();
        println!("Init fake disk success");
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
        println!("Init keyboard input device success");
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
        println!("Init mouse input device success");
    }
}

fn init_net() {
    // If we need run test, we should only init loop device because no we can't route packet
    #[cfg(feature = "test")]
    {
        init_loop_device();
    }
    #[cfg(not(feature = "test"))]
    {
        let res = crate::board::get_net_device_info();
        if res.is_none() {
            println!("There is no net device");
            return;
        }
        let (base_addr, irq) = res.unwrap();
        println!("Init net device, base_addr:{:#x},irq:{}", base_addr, irq);

        #[cfg(feature = "qemu")]
        {
            use crate::device::net::NetNeedFunc;
            use crate::driver::net::make_virtio_net_device;
            let virtio_net = make_virtio_net_device(base_addr);
            use core::str::FromStr;
            use crate::config::{QEMU_GATEWAY, QEMU_IP};
            let device = Box::new(virtio_net);
            netcore::init_net(
                device,
                Arc::new(NetNeedFunc),
                IpAddress::from_str(QEMU_IP).unwrap(),
            IpAddress::from_str(QEMU_GATEWAY).unwrap(),
                true
            );
            println!("Init net device success");
            #[cfg(feature = "net_test")]
            {
                println!("test echo-server");
                net::nettest::accept_loop();
            }
        }
    }
}
#[cfg(feature = "test")]
fn init_loop_device() {
    use crate::device::net::NetNeedFunc;
    use loopback::LoopbackDev;
    // use default ip and gateway for qemu
    let ip = IpAddress::v4(127,0,0,1);
    let gate_way = IpAddress::v4(127,0,0,1);
    let loopback = Box::new(LoopbackDev::new());
    netcore::init_net(
        loopback,
        Arc::new(NetNeedFunc),
        ip,
        gate_way,
        false,
    );
    println!("Init net device success");
}

