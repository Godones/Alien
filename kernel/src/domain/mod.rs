mod init;

extern crate alloc;
use alloc::{boxed::Box, sync::Arc, vec};

use domain_helper::{alloc_domain_id, SharedHeapAllocator};
use fdt::Fdt;
use interface::*;
use log::{info, warn};
use rref::{RRef, RRefVec};

use crate::{domain::init::init_domains, domain_helper, domain_loader::creator::*};

/// set the kernel to the specific domain
fn init_kernel_domain() {
    rref::init(Box::new(SharedHeapAllocator), alloc_domain_id());
}

fn init_device() -> Arc<dyn PLICDomain> {
    let devices = create_devices_domain("devices", None).unwrap();
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

    let plic = create_plic_domain("plic", None).unwrap();
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
                let rtc_driver = create_rtc_domain("goldfish", None).unwrap();
                rtc_driver.init(&device_info).unwrap();
                domain_helper::register_domain("goldfish", DomainType::RtcDomain(rtc_driver), true);
                let irq = device_info.irq as _;
                // todo!(register irq)
                plic.register_irq(irq, &RRefVec::from_slice("goldfish".as_bytes()))
                    .unwrap()
            }
            "uart" => {
                let uart_driver = create_uart_domain("uart16550", None).unwrap();
                uart_driver.init(&device_info).unwrap();
                domain_helper::register_domain("uart", DomainType::UartDomain(uart_driver), true);
                let buf_uart = create_buf_uart_domain("buf_uart", None).unwrap();
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
                let blk_driver = create_blk_device_domain("virtio_mmio_block", None).unwrap();
                blk_driver.init(&device_info).unwrap();
                info!(
                    "dev capacity: {:?}MB",
                    blk_driver.get_capacity().unwrap() / 1024 / 1024
                );
                domain_helper::register_domain(
                    "virtio_mmio_block",
                    DomainType::BlkDeviceDomain(blk_driver.clone()),
                    false,
                );

                let shadow_blk = create_shadow_block_domain("shadow_blk", None).unwrap();
                shadow_blk.init("virtio_mmio_block-1").unwrap();
                domain_helper::register_domain(
                    "shadow_blk",
                    DomainType::ShadowBlockDomain(shadow_blk),
                    false,
                );
                let cache_blk = create_cache_blk_device_domain("cache_blk", None).unwrap();
                cache_blk.init("shadow_blk-1").unwrap();
                domain_helper::register_domain(
                    "cache_blk",
                    DomainType::CacheBlkDeviceDomain(cache_blk),
                    false,
                );
            }
            "virtio-mmio-net" => {
                let net_driver = create_net_domain("virtio_mmio_net", None).unwrap();
                net_driver.init(&device_info).unwrap();
                domain_helper::register_domain(
                    "virtio_mmio_net",
                    DomainType::NetDeviceDomain(net_driver),
                    false,
                );
                // todo!(register irq)
                plic.register_irq(
                    irq as _,
                    &RRefVec::from_slice("virtio_mmio_net-1".as_bytes()),
                )
                .unwrap()
            }
            "virtio-mmio-input" => {
                let input_driver = create_input_domain("virtio_mmio_input", None).unwrap();
                input_driver.init(&device_info).unwrap();
                domain_helper::register_domain(
                    "virtio_mmio_input",
                    DomainType::InputDomain(input_driver),
                    false,
                );
                // todo!(register irq)
                plic.register_irq(
                    irq as _,
                    &RRefVec::from_slice("virtio_mmio_input-1".as_bytes()),
                )
                .unwrap()
            }
            #[cfg(feature = "gui")]
            "virtio-mmio-gpu" => {
                let gpu_driver = create_gpu_domain("virtio_mmio_gpu", None).unwrap();
                gpu_driver.init(&device_info).unwrap();
                domain_helper::register_domain(
                    "virtio_mmio_gpu",
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
        let null_device = create_empty_device_domain("null", None).unwrap();
        null_device.init().unwrap();
        domain_helper::register_domain("null", DomainType::EmptyDeviceDomain(null_device), true);
        let random_device = create_empty_device_domain("random", None).unwrap();
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
    init_domains();
    init_kernel_domain();
    domain_helper::init_domain_create(Box::new(DomainCreateImpl));

    let fatfs = create_fs_domain("fatfs", None).unwrap();
    domain_helper::register_domain("fatfs", DomainType::FsDomain(fatfs.clone()), false);

    let ramfs = create_fs_domain("ramfs", None).unwrap();
    domain_helper::register_domain("ramfs", DomainType::FsDomain(ramfs.clone()), false);

    let devfs = create_devfs_domain("devfs", None).unwrap();
    domain_helper::register_domain("devfs", DomainType::DevFsDomain(devfs.clone()), true);

    let procfs = create_fs_domain("procfs", None).unwrap();
    domain_helper::register_domain("procfs", DomainType::FsDomain(procfs.clone()), true);

    let sysfs = create_fs_domain("sysfs", None).unwrap();
    domain_helper::register_domain("sysfs", DomainType::FsDomain(sysfs.clone()), true);

    let pipefs = create_fs_domain("pipefs", None).unwrap();
    domain_helper::register_domain("pipefs", DomainType::FsDomain(pipefs.clone()), true);

    let vfs = create_vfs_domain("vfs", None).unwrap();
    domain_helper::register_domain("vfs", DomainType::VfsDomain(vfs.clone()), true);

    let task = create_task_domain("task", None).unwrap();
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

    let syscall = create_syscall_domain("syscall", None).unwrap();
    syscall.init().unwrap();
    domain_helper::register_domain("syscall", DomainType::SysCallDomain(syscall.clone()), true);

    platform::println!("Load domains done");

    crate::task::register_task_domain(task);
    crate::trap::register_syscall_domain(syscall);
    crate::trap::register_plic_domain(plic);
    platform::println!("Register task domain and syscall domain to trap system done");
}
