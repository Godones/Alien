mod create;

extern crate alloc;

use alloc::{boxed::Box, sync::Arc, vec};

use domain_helper::{alloc_domain_id, SharedHeapAllocator};
use fdt::Fdt;
use interface::*;
use log::{info, warn};
use rref::{RRef, RRefVec};

use crate::{
    domain::create::DomainCreateImpl, domain_helper, domain_loader::DomainLoader, domain_proxy::*,
};

#[macro_use]
mod macros {
    #[repr(C)] // guarantee 'bytes' comes after '_align'
    pub struct AlignedAs<Align, Bytes: ?Sized> {
        pub _align: [Align; 0],
        pub bytes: Bytes,
    }

    macro_rules! include_bytes_align_as {
        ($align_ty:ty, $path:literal) => {{
            // const block expression to encapsulate the static
            use $crate::domain::macros::AlignedAs;

            // this assignment is made possible by CoerceUnsized
            static ALIGNED: &AlignedAs<$align_ty, [u8]> = &AlignedAs {
                _align: [],
                bytes: *include_bytes!($path),
            };

            &ALIGNED.bytes
        }};
    }
}
static UART_DOMAIN: &'static [u8] =
    include_bytes_align_as!(usize, "../../../build/guart_domain.bin");
#[cfg(feature = "gui")]
static GPU_DOMAIN: &'static [u8] = include_bytes_align_as!(usize, "../../../build/ggpu_domain.bin");
static BLK_DOMAIN: &'static [u8] = include_bytes_align_as!(usize, "../../../build/gblk_domain.bin");

#[allow(unused)]
static FATFS_DOMAIN: &'static [u8] =
    include_bytes_align_as!(usize, "../../../build/gfatfs_domain.bin");
static RTC_DOMAIN: &'static [u8] =
    include_bytes_align_as!(usize, "../../../build/ggoldfish_domain.bin");
static VFS_DOMAIN: &'static [u8] = include_bytes_align_as!(usize, "../../../build/gvfs_domain.bin");
static CACHE_BLK_DOMAIN: &'static [u8] =
    include_bytes_align_as!(usize, "../../../build/gcache_blk_domain.bin");
static SHADOW_BLK_DOMAIN: &'static [u8] =
    include_bytes_align_as!(usize, "../../../build/gshadow_blk_domain.bin");

static EXTERN_INTR: &'static [u8] =
    include_bytes_align_as!(usize, "../../../build/gextern-interrupt_domain.bin");

static DEVICES_DOMAIN: &'static [u8] =
    include_bytes_align_as!(usize, "../../../build/gdevices_domain.bin");

static TASK_DOMAIN: &'static [u8] =
    include_bytes_align_as!(usize, "../../../build/gtask_domain.bin");

static SYS_CALL_DOMAIN: &'static [u8] =
    include_bytes_align_as!(usize, "../../../build/gsyscall_domain.bin");

static BUF_UART_DOMAIN: &'static [u8] =
    include_bytes_align_as!(usize, "../../../build/gbuf_uart_domain.bin");

static NET_DOMAIN: &'static [u8] =
    include_bytes_align_as!(usize, "../../../build/gvirtio-mmio-net_domain.bin");

static INPUT_DOMAIN: &'static [u8] =
    include_bytes_align_as!(usize, "../../../build/ginput_domain.bin");

static RAMFS_DOMAIN: &'static [u8] =
    include_bytes_align_as!(usize, "../../../build/gramfs_domain.bin");

static NULL_DOMAIN: &'static [u8] =
    include_bytes_align_as!(usize, "../../../build/gnull_domain.bin");

static RANDOM_DOMAIN: &'static [u8] =
    include_bytes_align_as!(usize, "../../../build/grandom_domain.bin");

static DEVFS_DOMAIN: &'static [u8] =
    include_bytes_align_as!(usize, "../../../build/gdevfs_domain.bin");

static PROCFS_DOMAIN: &'static [u8] =
    include_bytes_align_as!(usize, "../../../build/gprocfs_domain.bin");

static SYSFS_DOMAIN: &'static [u8] =
    include_bytes_align_as!(usize, "../../../build/gsysfs_domain.bin");

static PIPEFS_DOMAIN: &'static [u8] =
    include_bytes_align_as!(usize, "../../../build/gpipefs_domain.bin");

fn fatfs_domain() -> Arc<dyn FsDomain> {
    info!("Load fatfs domain, size: {}KB", FATFS_DOMAIN.len() / 1024);
    let mut domain = DomainLoader::new(FATFS_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let fatfs = domain.call(id);
    Arc::new(FsDomainProxy::new(id, fatfs))
}

fn ramfs_domain() -> Arc<dyn FsDomain> {
    info!("Load ramfs domain, size: {}KB", RAMFS_DOMAIN.len() / 1024);
    let mut domain = DomainLoader::new(RAMFS_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let ramfs = domain.call(id);
    Arc::new(FsDomainProxy::new(id, ramfs))
}

fn devfs_domain() -> Arc<dyn DevFsDomain> {
    info!("Load devfs domain, size: {}KB", DEVFS_DOMAIN.len() / 1024);
    let mut domain = DomainLoader::new(DEVFS_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let devfs = domain.call(id);
    Arc::new(DevFsDomainProxy::new(id, devfs))
}

fn uart_domain() -> Arc<dyn UartDomain> {
    info!("Loading uart domain, size: {}KB", UART_DOMAIN.len() / 1024);
    let mut domain = DomainLoader::new(UART_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let uart = domain.call(id);
    Arc::new(UartDomainProxy::new(id, uart))
}

#[cfg(feature = "gui")]
fn virtio_mmio_gpu_domain() -> Arc<dyn GpuDomain> {
    info!("Loading gpu domain, size: {}KB", GPU_DOMAIN.len() / 1024);
    let mut domain = DomainLoader::new(GPU_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let gpu = domain.call(id);
    Arc::new(GpuDomainProxy::new(id, gpu))
}

fn virtio_mmio_block_domain() -> Arc<dyn BlkDeviceDomain> {
    info!("Load blk domain, size: {}KB", BLK_DOMAIN.len() / 1024);
    let mut domain = DomainLoader::new(BLK_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let dev = domain.call(id);
    Arc::new(BlkDomainProxy::new(id, dev, domain))
}

fn goldfish_rtc_domain() -> Arc<dyn RtcDomain> {
    info!("Load rtc domain, size: {}KB", RTC_DOMAIN.len() / 1024);
    let mut domain = DomainLoader::new(RTC_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let rtc = domain.call(id);
    Arc::new(RtcDomainProxy::new(id, rtc))
}

fn vfs_domain() -> Arc<dyn VfsDomain> {
    info!("Load vfs domain, size: {}KB", VFS_DOMAIN.len() / 1024);
    let mut domain = DomainLoader::new(VFS_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let vfs = domain.call(id);
    Arc::new(VfsDomainProxy::new(id, vfs))
}

fn cache_blk_domain() -> Arc<dyn CacheBlkDeviceDomain> {
    info!(
        "Load cache blk domain, size: {}KB",
        CACHE_BLK_DOMAIN.len() / 1024
    );
    let mut domain = DomainLoader::new(CACHE_BLK_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let cache_blk = domain.call(id);
    Arc::new(CacheBlkDomainProxy::new(id, cache_blk))
}

fn shadow_blk_domain() -> Arc<dyn ShadowBlockDomain> {
    info!(
        "Load shadow blk domain, size: {}KB",
        SHADOW_BLK_DOMAIN.len() / 1024
    );
    let mut domain = DomainLoader::new(SHADOW_BLK_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let shadow_blk = domain.call(id);
    Arc::new(ShadowBlockDomainProxy::new(id, shadow_blk))
}

fn plic_domain() -> Arc<dyn PLICDomain> {
    info!(
        "Load extern-interrupt domain, size: {}KB",
        EXTERN_INTR.len() / 1024
    );
    let mut domain = DomainLoader::new(EXTERN_INTR);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let extern_intr_domain = domain.call(id);
    Arc::new(PLICDomainProxy::new(id, extern_intr_domain))
}

fn devices_domain() -> Arc<dyn DevicesDomain> {
    info!(
        "Load devices domain, size: {}KB",
        DEVICES_DOMAIN.len() / 1024
    );
    let mut domain = DomainLoader::new(DEVICES_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let devices_domain = domain.call(id);

    Arc::new(DevicesDomainProxy::new(id, devices_domain))
}

fn task_domain() -> Arc<dyn TaskDomain> {
    info!("Load task domain, size: {}KB", TASK_DOMAIN.len() / 1024);
    let mut domain = DomainLoader::new(TASK_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let task_domain = domain.call(id);
    Arc::new(TaskDomainProxy::new(id, task_domain))
}

fn syscall_domain() -> Arc<dyn SysCallDomain> {
    info!(
        "Load syscall domain, size: {}KB",
        SYS_CALL_DOMAIN.len() / 1024
    );
    let mut domain = DomainLoader::new(SYS_CALL_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let task_domain = domain.call(id);
    Arc::new(SysCallDomainProxy::new(id, task_domain))
}

fn buf_uart_domain() -> Arc<dyn BufUartDomain> {
    info!(
        "Load buf_uart domain, size: {}KB",
        BUF_UART_DOMAIN.len() / 1024
    );
    let mut domain = DomainLoader::new(BUF_UART_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let uart = domain.call(id);
    Arc::new(BufUartDomainProxy::new(id, uart))
}

fn virtio_mmio_net_domain() -> Arc<dyn NetDomain> {
    info!("Load net domain, size: {}KB", NET_DOMAIN.len() / 1024);
    let mut domain = DomainLoader::new(NET_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let net = domain.call(id);
    Arc::new(NetDomainProxy::new(id, net))
}

fn virtio_mmio_input_domain() -> Arc<dyn InputDomain> {
    info!("Load input domain, size: {} KB", INPUT_DOMAIN.len() / 1024);
    let mut domain = DomainLoader::new(INPUT_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let input = domain.call(id);
    Arc::new(InputDomainProxy::new(id, input))
}

fn null_device_domain() -> Arc<dyn EmptyDeviceDomain> {
    info!("Load null domain, size: {}KB", NULL_DOMAIN.len() / 1024);
    let mut domain = DomainLoader::new(NULL_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let null = domain.call(id);
    Arc::new(EmptyDeviceDomainProxy::new(id, null))
}

fn random_device_domain() -> Arc<dyn EmptyDeviceDomain> {
    info!("Load random domain, size: {}KB", RANDOM_DOMAIN.len() / 1024);
    let mut domain = DomainLoader::new(RANDOM_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let random = domain.call(id);
    Arc::new(EmptyDeviceDomainProxy::new(id, random))
}

fn procfs_domain() -> Arc<dyn FsDomain> {
    info!("Load procfs domain, size: {}KB", PROCFS_DOMAIN.len() / 1024);
    let mut domain = DomainLoader::new(PROCFS_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let procfs = domain.call(id);
    Arc::new(FsDomainProxy::new(id, procfs))
}
fn sysfs_domain() -> Arc<dyn FsDomain> {
    info!("Load sysfs domain, size: {}KB", SYSFS_DOMAIN.len() / 1024);
    let mut domain = DomainLoader::new(SYSFS_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let procfs = domain.call(id);
    Arc::new(FsDomainProxy::new(id, procfs))
}

fn pipefs_domain() -> Arc<dyn FsDomain> {
    info!("Load pipefs domain, size: {}KB", PIPEFS_DOMAIN.len() / 1024);
    let mut domain = DomainLoader::new(PIPEFS_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let pipefs = domain.call(id);
    Arc::new(FsDomainProxy::new(id, pipefs))
}

/// set the kernel to the specific domain
fn init_kernel_domain() {
    rref::init(Box::new(SharedHeapAllocator), alloc_domain_id());
}

fn init_device() -> Arc<dyn PLICDomain> {
    let devices = devices_domain();
    let ptr = platform::platform_dtb_ptr();
    let fdt = unsafe { Fdt::from_ptr(ptr as *const u8) }
        .unwrap()
        .raw_data();
    devices.init(fdt).unwrap();
    domain_helper::register_domain("devices", DomainType::DevicesDomain(devices.clone()), true);

    let mut device_info = RRef::new(DeviceInfo::default());
    let mut info = vec![];
    loop {
        let res = devices.index_device(device_info.next, device_info);
        if res.is_err() {
            panic!("index device error");
        }
        device_info = res.unwrap();
        if device_info.next == 0 {
            break;
        }
        info.push(device_info.clone());
        device_info.name.fill(0);
    }
    let plic_info = info.iter().find(|x| {
        let name_len = x.name.iter().position(|&x| x == 0).unwrap_or(x.name.len());
        let name = core::str::from_utf8(&x.name[..name_len]).unwrap();
        name == "plic"
    });

    let plic = plic_domain();
    match plic_info {
        Some(plic_info) => {
            plic.init(plic_info).unwrap();
            domain_helper::register_domain("plic", DomainType::PLICDomain(plic.clone()), true);
        }
        None => panic!("no plic device"),
    }

    for device_info in info {
        let name_len = device_info
            .name
            .iter()
            .position(|&x| x == 0)
            .unwrap_or(device_info.name.len());
        let name = core::str::from_utf8(&device_info.name[..name_len]).unwrap();
        let irq = device_info.irq;
        info!("device name: {}", name);
        // todo!(other match methods)
        match name {
            "rtc" => {
                let rtc_driver = goldfish_rtc_domain();
                rtc_driver.init(&device_info).unwrap();
                domain_helper::register_domain("rtc", DomainType::RtcDomain(rtc_driver), true);
                let irq = device_info.irq as _;
                // todo!(register irq)
                plic.register_irq(irq, &RRefVec::from_slice("rtc".as_bytes()))
                    .unwrap()
            }
            "uart" => {
                let uart_driver = uart_domain();
                uart_driver.init(&device_info).unwrap();
                domain_helper::register_domain("uart", DomainType::UartDomain(uart_driver), true);
                let buf_uart = buf_uart_domain();
                buf_uart.init("uart").unwrap();
                buf_uart.putc('U' as u8).unwrap();
                buf_uart.putc('A' as u8).unwrap();
                buf_uart.putc('R' as u8).unwrap();
                buf_uart.putc('T' as u8).unwrap();
                buf_uart.putc('\n' as u8).unwrap();
                domain_helper::register_domain(
                    "buf_uart",
                    DomainType::BufUartDomain(buf_uart),
                    true,
                );
                // todo!(register irq)
                plic.register_irq(irq as _, &RRefVec::from_slice("buf_uart".as_bytes()))
                    .unwrap()
            }
            "virtio-mmio-block" => {
                let blk_driver = virtio_mmio_block_domain();
                blk_driver.init(&device_info).unwrap();
                info!(
                    "dev capacity: {:?}MB",
                    blk_driver.get_capacity().unwrap() / 1024 / 1024
                );
                domain_helper::register_domain(
                    "virtio-mmio-block",
                    DomainType::BlkDeviceDomain(blk_driver.clone()),
                    false,
                );

                let shadow_blk = shadow_blk_domain();
                shadow_blk.init("virtio-mmio-block-1").unwrap();
                domain_helper::register_domain(
                    "shadow_blk",
                    DomainType::ShadowBlockDomain(shadow_blk),
                    false,
                );
                let cache_blk = cache_blk_domain();
                cache_blk.init("shadow_blk-1").unwrap();
                domain_helper::register_domain(
                    "cache_blk",
                    DomainType::CacheBlkDeviceDomain(cache_blk),
                    false,
                );
            }
            "virtio-mmio-net" => {
                let net_driver = virtio_mmio_net_domain();
                net_driver.init(&device_info).unwrap();
                domain_helper::register_domain(
                    "virtio-mmio-net",
                    DomainType::NetDomain(net_driver),
                    false,
                );
                // todo!(register irq)
                plic.register_irq(
                    irq as _,
                    &RRefVec::from_slice("virtio-mmio-net-1".as_bytes()),
                )
                .unwrap()
            }
            "virtio-mmio-input" => {
                let input_driver = virtio_mmio_input_domain();
                input_driver.init(&device_info).unwrap();
                domain_helper::register_domain(
                    "virtio-mmio-input",
                    DomainType::InputDomain(input_driver),
                    false,
                );
                // todo!(register irq)
                plic.register_irq(
                    irq as _,
                    &RRefVec::from_slice("virtio-mmio-input-1".as_bytes()),
                )
                .unwrap()
            }
            #[cfg(feature = "gui")]
            "virtio-mmio-gpu" => {
                let gpu_driver = virtio_mmio_gpu_domain();
                gpu_driver.init(&device_info).unwrap();
                domain_helper::register_domain(
                    "virtio-mmio-gpu",
                    DomainType::GpuDomain(gpu_driver),
                    false,
                );
            }
            _ => {
                warn!("unknown device: {}", name);
            }
        }
    }

    {
        let null_device = null_device_domain();
        null_device.init().unwrap();
        domain_helper::register_domain("null", DomainType::EmptyDeviceDomain(null_device), true);
        let random_device = random_device_domain();
        random_device.init().unwrap();
        domain_helper::register_domain(
            "random",
            DomainType::EmptyDeviceDomain(random_device),
            true,
        );
    }

    plic
}

pub fn load_domains() {
    init_kernel_domain();
    domain_helper::init_domain_create(Box::new(DomainCreateImpl));

    let fatfs = fatfs_domain();
    domain_helper::register_domain("fatfs", DomainType::FsDomain(fatfs.clone()), false);

    let ramfs = ramfs_domain();
    domain_helper::register_domain("ramfs", DomainType::FsDomain(ramfs.clone()), false);

    let devfs = devfs_domain();
    domain_helper::register_domain("devfs", DomainType::DevFsDomain(devfs.clone()), true);

    let procfs = procfs_domain();
    domain_helper::register_domain("procfs", DomainType::FsDomain(procfs.clone()), true);

    let sysfs = sysfs_domain();
    domain_helper::register_domain("sysfs", DomainType::FsDomain(sysfs.clone()), true);

    let pipefs = pipefs_domain();
    domain_helper::register_domain("pipefs", DomainType::FsDomain(pipefs.clone()), true);

    let vfs = vfs_domain();
    domain_helper::register_domain("vfs", DomainType::VfsDomain(vfs.clone()), true);

    let task = task_domain();
    domain_helper::register_domain("task", DomainType::TaskDomain(task.clone()), true);

    // we need to register vfs and task domain before init device, because we need to use vfs and task domain in some
    // device init function
    let plic = init_device();

    devfs.init().unwrap();
    fatfs.init().unwrap();
    ramfs.init().unwrap();
    procfs.init().unwrap();
    sysfs.init().unwrap();

    // The vfs domain may use the device domain, so we need to init vfs domain after init device domain,
    // also it may use the task domain.
    vfs.init().unwrap();
    task.init().unwrap();

    let syscall = syscall_domain();
    syscall.init().unwrap();
    domain_helper::register_domain("syscall", DomainType::SysCallDomain(syscall.clone()), true);

    platform::println!("Load domains done");

    crate::task::register_task_domain(task);
    crate::trap::register_syscall_domain(syscall);
    crate::trap::register_plic_domain(plic);
    platform::println!("Register task domain and syscall domain to trap system done");
}
