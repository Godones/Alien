mod init;

extern crate alloc;
use alloc::{boxed::Box, sync::Arc};

use basic::bus::mmio::VirtioMmioDeviceType;
use domain_helper::{alloc_domain_id, SharedHeapAllocator};
use interface::*;
use log::{info, warn};
use rref::RRefVec;

use crate::{
    domain::init::init_domains, domain_helper, domain_loader::creator::*, mmio_bus, platform_bus,
};

/// set the kernel to the specific domain
fn init_kernel_domain() {
    rref::init(Box::new(SharedHeapAllocator), alloc_domain_id());
}

fn init_device() -> Arc<dyn PLICDomain> {
    let platform_bus = platform_bus!();

    let plic_device = platform_bus
        .common_devices()
        .iter()
        .find(|device| device.name() == "plic")
        .expect("plic device not found");

    let plic = create_plic_domain("plic", None).unwrap();
    let plic_address = plic_device.address().as_usize();
    let plic_size = plic_device.io_region().size();
    plic.init(plic_address..plic_address + plic_size).unwrap();
    domain_helper::register_domain("plic", DomainType::PLICDomain(plic.clone()), true);

    platform_bus.common_devices().iter().for_each(|device| {
        let address = device.address().as_usize();
        let size = device.io_region().size();
        let irq = device.irq();
        match device.name() {
            "rtc" => {
                let rtc = create_rtc_domain("goldfish", None).unwrap();
                rtc.init(address..address + size).unwrap();
                domain_helper::register_domain("rtc", DomainType::RtcDomain(rtc.clone()), true);
                plic.register_irq(irq.unwrap() as _, &RRefVec::from_slice("rtc".as_bytes()))
                    .unwrap();
            }
            "uart" => {
                let uart = create_uart_domain("uart16550", None).unwrap();
                uart.init(address..address + size).unwrap();
                domain_helper::register_domain("uart", DomainType::UartDomain(uart.clone()), true);
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
                plic.register_irq(
                    irq.unwrap() as _,
                    &RRefVec::from_slice("buf_uart".as_bytes()),
                )
                .unwrap();
            }
            _ => {
                warn!("unknown device: {}", device.name());
            }
        }
    });

    mmio_bus!()
        .lock()
        .common_devices()
        .iter()
        .for_each(|device| {
            let address = device.address().as_usize();
            let size = device.io_region().size();
            let _irq = device.irq();
            match device.device_type() {
                VirtioMmioDeviceType::Network => {
                    let net_driver = create_net_domain("virtio_mmio_net", None).unwrap();
                    net_driver.init(address..address + size).unwrap();
                    domain_helper::register_domain(
                        "virtio_mmio_net",
                        DomainType::NetDeviceDomain(net_driver.clone()),
                        false,
                    );
                    // register irq
                    // plic.register_irq(
                    //     irq.unwrap() as _,
                    //     &RRefVec::from_slice("virtio_mmio_net-1".as_bytes()),
                    // )
                    // .unwrap()
                }
                VirtioMmioDeviceType::Block => {
                    let blk_driver = create_blk_device_domain("virtio_mmio_block", None).unwrap();
                    blk_driver.init(address..address + size).unwrap();
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
                    // register irq
                }
                VirtioMmioDeviceType::Input => {
                    let input_driver = create_input_domain("virtio_mmio_input", None);
                    if let Some(input_driver) = input_driver {
                        input_driver.init(address..address + size).unwrap();
                        domain_helper::register_domain(
                            "virtio_mmio_input",
                            DomainType::InputDomain(input_driver),
                            false,
                        );
                    }
                    // register irq
                    // plic.register_irq(
                    //     irq.unwrap() as _,
                    //     &RRefVec::from_slice("virtio_mmio_input-1".as_bytes()),
                    // )
                    //     .unwrap()
                }
                VirtioMmioDeviceType::GPU => {
                    let gpu_driver = create_gpu_domain("virtio_mmio_gpu", None);
                    if let Some(gpu_driver) = gpu_driver {
                        gpu_driver.init(address..address + size).unwrap();
                        domain_helper::register_domain(
                            "virtio_mmio_gpu",
                            DomainType::GpuDomain(gpu_driver),
                            false,
                        );
                    }
                }
                _ => {
                    warn!("unknown device: {:?}", device.device_type());
                }
            }
        });

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

    let scheduler = create_scheduler_domain("fifo_scheduler", None).unwrap();
    domain_helper::register_domain(
        "scheduler",
        DomainType::SchedulerDomain(scheduler.clone()),
        true,
    );

    let task = create_task_domain("task", None).unwrap(); // ref to scheduler domain
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
    {
        let initrd = mem::INITRD_DATA.lock();
        let data = initrd.as_ref().unwrap();
        vfs.init(data.as_slice()).unwrap();
    }

    scheduler.init().unwrap();
    task.init().unwrap();

    let syscall = create_syscall_domain("syscall", None).unwrap();
    syscall.init().unwrap();
    domain_helper::register_domain("syscall", DomainType::SysCallDomain(syscall.clone()), true);

    platform::println!("Load domains done");

    crate::task::register_scheduler_domain(scheduler);
    crate::task::register_task_domain(task);
    crate::trap::register_syscall_domain(syscall);
    crate::trap::register_plic_domain(plic);
    platform::println!("Register task domain and syscall domain to trap system done");
}
