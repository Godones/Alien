mod init;

extern crate alloc;
use alloc::{boxed::Box, string::ToString, sync::Arc};

use basic::bus::mmio::VirtioMmioDeviceType;
use corelib::AlienResult;
use domain_helper::alloc_domain_id;
use interface::*;
use log::warn;
use rref::RRefVec;

use crate::{
    create_domain,
    domain::init::init_domains,
    domain_helper,
    domain_helper::{DOMAIN_DATA_ALLOCATOR, SHARED_HEAP_ALLOCATOR},
    domain_loader::creator::*,
    domain_proxy::*,
    mmio_bus, platform_bus, register_domain,
};

/// set the kernel to the specific domain
fn init_kernel_domain() {
    rref::init(SHARED_HEAP_ALLOCATOR, alloc_domain_id());
    storage::init_data_allocator(DOMAIN_DATA_ALLOCATOR);
}

fn init_device() -> AlienResult<Arc<dyn PLICDomain>> {
    let platform_bus = platform_bus!();

    let plic_device = platform_bus
        .common_devices()
        .iter()
        .find(|device| device.name() == "plic")
        .expect("plic device not found");

    let (plic, domain_file_info) =
        create_domain!(PLICDomainProxy, DomainTypeRaw::PLICDomain, "plic")?;
    let plic_address = plic_device.address_range();
    let plic_info = PlicInfo {
        device_info: plic_address.start.as_usize()..plic_address.end.as_usize(),
        #[cfg(qemu_riscv)]
        ty: PlicType::Qemu,
        #[cfg(vf2)]
        ty: PlicType::SiFive,
    };
    plic.init_by_box(Box::new(plic_info))?;
    register_domain!(
        "plic",
        domain_file_info,
        DomainType::PLICDomain(plic.clone()),
        true
    );

    let mut nic_irq = 0;

    for device in platform_bus.common_devices().iter() {
        let address = device.address().as_usize();
        let size = device.io_region().size();
        let irq = device.irq();
        match device.name() {
            "rtc" => {
                if let Some(compatible) = device.compatible() {
                    if compatible != "google,goldfish-rtc" {
                        println_color!(31, "unknown rtc device: {}", compatible);
                        continue;
                    }
                }
                let (rtc, domain_file_info) =
                    create_domain!(RtcDomainProxy, DomainTypeRaw::RtcDomain, "goldfish")?;
                rtc.init_by_box(Box::new(address..address + size))?;
                register_domain!("rtc", domain_file_info, DomainType::RtcDomain(rtc), true);
                plic.register_irq(irq.unwrap() as _, &RRefVec::from_slice("rtc".as_bytes()))?;
            }
            "uart" => {
                let compatible = device
                    .compatible()
                    .expect("uart device must have compatible property");
                let (uart, domain_file_info) = match compatible {
                    "ns16550a" => {
                        create_domain!(UartDomainProxy, DomainTypeRaw::UartDomain, "uart16550")?
                    }
                    "snps,dw-apb-uart" => {
                        create_domain!(UartDomainProxy, DomainTypeRaw::UartDomain, "uart8250")?
                    }
                    _ => panic!("unknown uart device: {}", compatible),
                };

                uart.init_by_box(Box::new(address..address + size))?;
                register_domain!("uart", domain_file_info, DomainType::UartDomain(uart), true);
                let (buf_uart, domain_file_info) =
                    create_domain!(BufUartDomainProxy, DomainTypeRaw::BufUartDomain, "buf_uart")?;
                buf_uart.init_by_box(Box::new("uart".to_string()))?;
                register_domain!(
                    "buf_uart",
                    domain_file_info,
                    DomainType::BufUartDomain(buf_uart),
                    true
                );
                plic.register_irq(
                    irq.unwrap() as _,
                    &RRefVec::from_slice("buf_uart".as_bytes()),
                )?;
            }
            "ramdisk" => {
                let (ramdisk, domain_file_info) =
                    create_domain!(BlkDomainProxy, DomainTypeRaw::BlkDeviceDomain, "mem_block")?;
                ramdisk.init_by_box(Box::new(address..address + size))?;
                register_domain!(
                    "block",
                    domain_file_info,
                    DomainType::BlkDeviceDomain(ramdisk),
                    false
                );
            }
            "loopback" => {
                let (net_driver, domain_file_info) = create_domain!(
                    NetDeviceDomainProxy,
                    DomainTypeRaw::NetDeviceDomain,
                    "loopback"
                )?;
                net_driver.init_by_box(Box::new(address..address + size))?;
                register_domain!(
                    "nic",
                    domain_file_info,
                    DomainType::NetDeviceDomain(net_driver),
                    false
                );
                let irq = device.irq();
                nic_irq = irq.unwrap();
            }
            "sdcard" => {
                let (sdcard, domain_file_info) =
                    create_domain!(BlkDomainProxy, DomainTypeRaw::BlkDeviceDomain, "vf2_sd")?;
                sdcard.init_by_box(Box::new(address..address + size))?;
                register_domain!(
                    "block",
                    domain_file_info,
                    DomainType::BlkDeviceDomain(sdcard),
                    false
                );
            }
            _ => {
                warn!("unknown device: {}", device.name());
            }
        }
    }

    for device in mmio_bus!().lock().common_devices().iter() {
        let address = device.address().as_usize();
        let size = device.io_region().size();
        match device.device_type() {
            VirtioMmioDeviceType::Network => {
                let (net_driver, domain_file_info) = create_domain!(
                    NetDeviceDomainProxy,
                    DomainTypeRaw::NetDeviceDomain,
                    "virtio_mmio_net"
                )?;
                net_driver.init_by_box(Box::new(address..address + size))?;
                register_domain!(
                    "nic",
                    domain_file_info,
                    DomainType::NetDeviceDomain(net_driver),
                    false
                );
                let irq = device.irq();
                nic_irq = irq.unwrap();
            }
            VirtioMmioDeviceType::Block => {
                let (blk_driver, domain_file_info) = create_domain!(
                    BlkDomainProxy,
                    DomainTypeRaw::BlkDeviceDomain,
                    "virtio_mmio_block"
                )?;
                blk_driver.init_by_box(Box::new(address..address + size))?;
                println!(
                    "dev capacity: {:?}MB",
                    blk_driver.get_capacity()? / 1024 / 1024
                );
                register_domain!(
                    "block",
                    domain_file_info,
                    DomainType::BlkDeviceDomain(blk_driver),
                    false
                );
                // register irq
            }
            VirtioMmioDeviceType::Input => {
                let (input_driver, domain_file_info) = create_domain!(
                    InputDomainProxy,
                    DomainTypeRaw::InputDomain,
                    "virtio_mmio_input"
                )?;
                input_driver.init_by_box(Box::new(address..address + size))?;
                let input_name = register_domain!(
                    "virtio_mmio_input",
                    domain_file_info,
                    DomainType::InputDomain(input_driver),
                    false
                );
                let (buf_input, domain_file_info) = create_domain!(
                    BufInputDomainProxy,
                    DomainTypeRaw::BufInputDomain,
                    "buf_input"
                )?;
                assert!(input_name.starts_with("virtio_mmio_input-"));
                buf_input.init_by_box(Box::new(input_name))?;
                let buf_input_name = register_domain!(
                    "buf_input",
                    domain_file_info,
                    DomainType::BufInputDomain(buf_input),
                    false
                );
                assert!(buf_input_name.starts_with("buf_input-"));
                // register irq
                let irq = device.irq();
                plic.register_irq(
                    irq.unwrap() as _,
                    &RRefVec::from_slice(buf_input_name.as_bytes()),
                )?
            }
            VirtioMmioDeviceType::GPU => {
                let (gpu_driver, domain_file_info) =
                    create_domain!(GpuDomainProxy, DomainTypeRaw::GpuDomain, "virtio_mmio_gpu")?;
                gpu_driver.init_by_box(Box::new(address..address + size))?;
                register_domain!(
                    "virtio_mmio_gpu",
                    domain_file_info,
                    DomainType::GpuDomain(gpu_driver),
                    true
                );
            }
            _ => {
                warn!("unknown device: {:?}", device.device_type());
            }
        }
    }
    {
        let (net_stack, domain_file_info) =
            create_domain!(NetDomainProxy, DomainTypeRaw::NetDomain, "net_stack")?;
        net_stack.init_by_box(Box::new("nic-1".to_string()))?;
        register_domain!(
            "net_stack",
            domain_file_info,
            DomainType::NetDomain(net_stack),
            true
        );
        // register irq
        plic.register_irq(nic_irq as _, &RRefVec::from_slice("net_stack".as_bytes()))?
    }
    // create shadow block and cache block device
    {
        let (shadow_blk, domain_file_info) = create_domain!(
            ShadowBlockDomainProxy,
            DomainTypeRaw::ShadowBlockDomain,
            "shadow_blk"
        )?;
        shadow_blk.init_by_box(Box::new("block-1".to_string()))?;
        register_domain!(
            "shadow_blk",
            domain_file_info,
            DomainType::ShadowBlockDomain(shadow_blk),
            false
        );
        let (cache_blk, domain_file_info) = create_domain!(
            CacheBlkDomainProxy,
            DomainTypeRaw::CacheBlkDeviceDomain,
            "cache_blk"
        )?;
        cache_blk.init_by_box(Box::new("shadow_blk-1".to_string()))?;
        register_domain!(
            "cache_blk",
            domain_file_info,
            DomainType::CacheBlkDeviceDomain(cache_blk),
            false
        );
    }

    // create random and null device
    {
        let (null_device, domain_file_info) = create_domain!(
            EmptyDeviceDomainProxy,
            DomainTypeRaw::EmptyDeviceDomain,
            "null"
        )?;
        null_device.init_by_box(Box::new(()))?;
        register_domain!(
            "null",
            domain_file_info,
            DomainType::EmptyDeviceDomain(null_device),
            true
        );
        let (random_device, domain_file_info) = create_domain!(
            EmptyDeviceDomainProxy,
            DomainTypeRaw::EmptyDeviceDomain,
            "random"
        )?;
        random_device.init_by_box(Box::new(()))?;
        register_domain!(
            "random",
            domain_file_info,
            DomainType::EmptyDeviceDomain(random_device),
            true
        );
    }
    Ok(plic)
}

pub fn load_domains() -> AlienResult<()> {
    init_domains();
    init_kernel_domain();
    domain_helper::init_domain_create(Box::new(DomainCreateImpl));

    let (scheduler, domain_file_info) = create_domain!(
        SchedulerDomainProxy,
        DomainTypeRaw::SchedulerDomain,
        "fifo_scheduler"
    )?;
    scheduler.init_by_box(Box::new(()))?;
    register_domain!(
        "scheduler",
        domain_file_info,
        DomainType::SchedulerDomain(scheduler.clone()),
        true
    );
    crate::task::register_scheduler_domain(scheduler);

    let (logger, domain_file_info) =
        create_domain!(LogDomainProxy, DomainTypeRaw::LogDomain, "logger")?;
    logger.init_by_box(Box::new(()))?;
    register_domain!(
        "logger",
        domain_file_info,
        DomainType::LogDomain(logger),
        true
    );

    let (fatfs, domain_file_info) =
        create_domain!(FsDomainProxy, DomainTypeRaw::FsDomain, "fatfs")?;
    register_domain!(
        "fatfs",
        domain_file_info,
        DomainType::FsDomain(fatfs.clone()),
        false
    );

    let (ramfs, domain_file_info) =
        create_domain!(FsDomainProxy, DomainTypeRaw::FsDomain, "ramfs")?;
    register_domain!(
        "ramfs",
        domain_file_info,
        DomainType::FsDomain(ramfs.clone()),
        false
    );

    let (devfs, domain_file_info) =
        create_domain!(DevFsDomainProxy, DomainTypeRaw::DevFsDomain, "devfs")?;
    register_domain!(
        "devfs",
        domain_file_info,
        DomainType::DevFsDomain(devfs.clone()),
        true
    );

    let (procfs, domain_file_info) =
        create_domain!(FsDomainProxy, DomainTypeRaw::FsDomain, "procfs")?;
    register_domain!(
        "procfs",
        domain_file_info,
        DomainType::FsDomain(procfs.clone()),
        true
    );

    let (sysfs, domain_file_info) =
        create_domain!(FsDomainProxy, DomainTypeRaw::FsDomain, "sysfs")?;
    register_domain!(
        "sysfs",
        domain_file_info,
        DomainType::FsDomain(sysfs.clone()),
        true
    );

    let (pipefs, domain_file_info) =
        create_domain!(FsDomainProxy, DomainTypeRaw::FsDomain, "pipefs")?;
    register_domain!(
        "pipefs",
        domain_file_info,
        DomainType::FsDomain(pipefs.clone()),
        true
    );

    let (domainfs, domain_file_info) =
        create_domain!(FsDomainProxy, DomainTypeRaw::FsDomain, "domainfs")?;
    register_domain!(
        "domainfs",
        domain_file_info,
        DomainType::FsDomain(domainfs.clone()),
        true
    );

    let (vfs, domain_file_info) = create_domain!(VfsDomainProxy, DomainTypeRaw::VfsDomain, "vfs")?;
    register_domain!(
        "vfs",
        domain_file_info,
        DomainType::VfsDomain(vfs.clone()),
        true
    );

    let (task, domain_file_info) =
        create_domain!(TaskDomainProxy, DomainTypeRaw::TaskDomain, "task")?; // ref to scheduler domain
    register_domain!(
        "task",
        domain_file_info,
        DomainType::TaskDomain(task.clone()),
        true
    );

    // we need to register vfs and task domain before init device, because we need to use vfs and task domain in some
    // device init function
    let plic = init_device()?;

    devfs.init_by_box(Box::new(()))?;
    fatfs.init_by_box(Box::new(()))?;
    ramfs.init_by_box(Box::new(()))?;
    procfs.init_by_box(Box::new(()))?;
    sysfs.init_by_box(Box::new(()))?;
    domainfs.init_by_box(Box::new(()))?;

    // The vfs domain may use the device domain, so we need to init vfs domain after init device domain,
    // also it may use the task domain.
    {
        let mut initrd = mem::INITRD_DATA.lock();
        let data = initrd.as_ref().unwrap();
        vfs.init_by_box(Box::new(data.as_slice().to_vec()))?;
        initrd.take(); // release the initrd data
    }

    task.init_by_box(Box::new(()))?;

    let (syscall, domain_file_info) =
        create_domain!(SysCallDomainProxy, DomainTypeRaw::SysCallDomain, "syscall")?;
    syscall.init_by_box(Box::new(()))?;
    register_domain!(
        "syscall",
        domain_file_info,
        DomainType::SysCallDomain(syscall.clone()),
        true
    );

    platform::println!("Load domains done");

    crate::task::register_task_domain(task);
    crate::trap::register_syscall_domain(syscall);
    crate::trap::register_plic_domain(plic);
    platform::println!("Register task domain and syscall domain to trap system");
    Ok(())
}
