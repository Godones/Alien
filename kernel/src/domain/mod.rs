mod init;

extern crate alloc;
use alloc::{boxed::Box, sync::Arc};
use core::sync::atomic::AtomicUsize;

use basic::bus::mmio::VirtioMmioDeviceType;
use corelib::AlienResult;
use domain_helper::{alloc_domain_id, SharedHeapAllocator};
use interface::*;
use log::{info, warn};
use rref::RRefVec;

use crate::{
    create_domain,
    domain::init::init_domains,
    domain_helper,
    domain_loader::creator::*,
    domain_proxy::{
        BlkDomainProxy, BufInputDomainProxy, BufUartDomainProxy, CacheBlkDomainProxy,
        DevFsDomainProxy, EmptyDeviceDomainProxy, FsDomainProxy, GpuDomainProxy, InputDomainProxy,
        LogDomainProxy, NetDeviceDomainProxy, NetDomainProxy, PLICDomainProxy, RtcDomainProxy,
        SchedulerDomainProxy, ShadowBlockDomainProxy, SysCallDomainProxy, TaskDomainProxy,
        UartDomainProxy, VfsDomainProxy,
    },
    mmio_bus, platform_bus,
};

/// set the kernel to the specific domain
fn init_kernel_domain() {
    rref::init(Box::new(SharedHeapAllocator), alloc_domain_id());
}

fn init_device() -> AlienResult<Arc<dyn PLICDomain>> {
    let platform_bus = platform_bus!();

    let plic_device = platform_bus
        .common_devices()
        .iter()
        .find(|device| device.name() == "plic")
        .expect("plic device not found");

    let plic = create_domain!(PLICDomainProxy, DomainTypeRaw::PLICDomain, "plic")?;
    let plic_address = plic_device.address().as_usize();
    let plic_size = plic_device.io_region().size();
    plic.init(plic_address..plic_address + plic_size)?;
    domain_helper::register_domain("plic", DomainType::PLICDomain(plic.clone()), true);

    for device in platform_bus.common_devices().iter() {
        let address = device.address().as_usize();
        let size = device.io_region().size();
        let irq = device.irq();
        match device.name() {
            "rtc" => {
                let rtc = create_domain!(RtcDomainProxy, DomainTypeRaw::RtcDomain, "goldfish")?;
                rtc.init(address..address + size)?;
                domain_helper::register_domain("rtc", DomainType::RtcDomain(rtc.clone()), true);
                plic.register_irq(irq.unwrap() as _, &RRefVec::from_slice("rtc".as_bytes()))?;
            }
            "uart" => {
                let uart = create_domain!(UartDomainProxy, DomainTypeRaw::UartDomain, "uart16550")?;
                uart.init(address..address + size)?;
                domain_helper::register_domain("uart", DomainType::UartDomain(uart.clone()), true);
                let buf_uart =
                    create_domain!(BufUartDomainProxy, DomainTypeRaw::BufUartDomain, "buf_uart")?;
                buf_uart.init("uart")?;
                buf_uart.putc('U' as u8)?;
                buf_uart.putc('A' as u8)?;
                buf_uart.putc('R' as u8)?;
                buf_uart.putc('T' as u8)?;
                buf_uart.putc('\n' as u8)?;
                domain_helper::register_domain(
                    "buf_uart",
                    DomainType::BufUartDomain(buf_uart),
                    true,
                );
                plic.register_irq(
                    irq.unwrap() as _,
                    &RRefVec::from_slice("buf_uart".as_bytes()),
                )?;
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
                static NET_COUNT: AtomicUsize = AtomicUsize::new(0);
                NET_COUNT.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
                if NET_COUNT.load(core::sync::atomic::Ordering::SeqCst) > 1 {
                    warn!("only support one network device");
                    continue;
                }

                let net_driver = create_domain!(
                    NetDeviceDomainProxy,
                    DomainTypeRaw::NetDeviceDomain,
                    "virtio_mmio_net"
                )?;
                net_driver.init(address..address + size)?;
                domain_helper::register_domain(
                    "virtio_mmio_net",
                    DomainType::NetDeviceDomain(net_driver.clone()),
                    false,
                );
                let irq = device.irq();

                let net_stack =
                    create_domain!(NetDomainProxy, DomainTypeRaw::NetDomain, "net_stack")?;
                net_stack.init("virtio_mmio_net-1")?;
                domain_helper::register_domain("net_stack", DomainType::NetDomain(net_stack), true);

                // register irq
                plic.register_irq(
                    irq.unwrap() as _,
                    &RRefVec::from_slice("net_stack".as_bytes()),
                )?
            }
            VirtioMmioDeviceType::Block => {
                let blk_driver = create_domain!(
                    BlkDomainProxy,
                    DomainTypeRaw::BlkDeviceDomain,
                    "virtio_mmio_block"
                )?;
                blk_driver.init(address..address + size)?;
                info!(
                    "dev capacity: {:?}MB",
                    blk_driver.get_capacity()? / 1024 / 1024
                );
                domain_helper::register_domain(
                    "virtio_mmio_block",
                    DomainType::BlkDeviceDomain(blk_driver.clone()),
                    false,
                );

                let shadow_blk = create_domain!(
                    ShadowBlockDomainProxy,
                    DomainTypeRaw::ShadowBlockDomain,
                    "shadow_blk"
                )?;
                shadow_blk.init("virtio_mmio_block-1")?;
                domain_helper::register_domain(
                    "shadow_blk",
                    DomainType::ShadowBlockDomain(shadow_blk),
                    false,
                );
                let cache_blk = create_domain!(
                    CacheBlkDomainProxy,
                    DomainTypeRaw::CacheBlkDeviceDomain,
                    "cache_blk"
                )?;
                cache_blk.init("shadow_blk-1")?;
                domain_helper::register_domain(
                    "cache_blk",
                    DomainType::CacheBlkDeviceDomain(cache_blk),
                    false,
                );
                // register irq
            }
            VirtioMmioDeviceType::Input => {
                let input_driver = create_domain!(
                    InputDomainProxy,
                    DomainTypeRaw::InputDomain,
                    "virtio_mmio_input"
                )?;
                input_driver.init(address..address + size)?;
                domain_helper::register_domain(
                    "virtio_mmio_input",
                    DomainType::InputDomain(input_driver),
                    true,
                );
                let buf_input = create_domain!(
                    BufInputDomainProxy,
                    DomainTypeRaw::BufInputDomain,
                    "buf_input"
                )?;
                buf_input.init("virtio_mmio_input")?;
                domain_helper::register_domain(
                    "buf_input",
                    DomainType::BufInputDomain(buf_input),
                    true,
                );
                // register irq
                let irq = device.irq();
                plic.register_irq(
                    irq.unwrap() as _,
                    &RRefVec::from_slice("buf_input".as_bytes()),
                )?
            }
            VirtioMmioDeviceType::GPU => {
                let gpu_driver =
                    create_domain!(GpuDomainProxy, DomainTypeRaw::GpuDomain, "virtio_mmio_gpu")?;
                gpu_driver.init(address..address + size)?;
                domain_helper::register_domain(
                    "virtio_mmio_gpu",
                    DomainType::GpuDomain(gpu_driver),
                    false,
                );
            }
            _ => {
                warn!("unknown device: {:?}", device.device_type());
            }
        }
    }
    {
        let null_device = create_domain!(
            EmptyDeviceDomainProxy,
            DomainTypeRaw::EmptyDeviceDomain,
            "null"
        )?;
        null_device.init()?;
        domain_helper::register_domain("null", DomainType::EmptyDeviceDomain(null_device), true);
        let random_device = create_domain!(
            EmptyDeviceDomainProxy,
            DomainTypeRaw::EmptyDeviceDomain,
            "random"
        )?;
        random_device.init()?;
        domain_helper::register_domain(
            "random",
            DomainType::EmptyDeviceDomain(random_device),
            true,
        );
    }
    Ok(plic)
}

pub fn load_domains() -> AlienResult<()> {
    init_domains();
    init_kernel_domain();
    domain_helper::init_domain_create(Box::new(DomainCreateImpl));

    let scheduler = create_domain!(
        SchedulerDomainProxy,
        DomainTypeRaw::SchedulerDomain,
        "fifo_scheduler"
    )?;
    scheduler.init()?;
    domain_helper::register_domain(
        "scheduler",
        DomainType::SchedulerDomain(scheduler.clone()),
        true,
    );
    crate::task::register_scheduler_domain(scheduler);

    let logger = create_domain!(LogDomainProxy, DomainTypeRaw::LogDomain, "logger")?;
    logger.init()?;
    domain_helper::register_domain("logger", DomainType::LogDomain(logger), true);

    let fatfs = create_domain!(FsDomainProxy, DomainTypeRaw::FsDomain, "fatfs")?;
    domain_helper::register_domain("fatfs", DomainType::FsDomain(fatfs.clone()), false);

    let ramfs = create_domain!(FsDomainProxy, DomainTypeRaw::FsDomain, "ramfs")?;
    domain_helper::register_domain("ramfs", DomainType::FsDomain(ramfs.clone()), false);

    let devfs = create_domain!(DevFsDomainProxy, DomainTypeRaw::DevFsDomain, "devfs")?;
    domain_helper::register_domain("devfs", DomainType::DevFsDomain(devfs.clone()), true);

    let procfs = create_domain!(FsDomainProxy, DomainTypeRaw::FsDomain, "procfs")?;
    domain_helper::register_domain("procfs", DomainType::FsDomain(procfs.clone()), true);

    let sysfs = create_domain!(FsDomainProxy, DomainTypeRaw::FsDomain, "sysfs")?;
    domain_helper::register_domain("sysfs", DomainType::FsDomain(sysfs.clone()), true);

    let pipefs = create_domain!(FsDomainProxy, DomainTypeRaw::FsDomain, "pipefs")?;
    domain_helper::register_domain("pipefs", DomainType::FsDomain(pipefs.clone()), true);

    let vfs = create_domain!(VfsDomainProxy, DomainTypeRaw::VfsDomain, "vfs")?;
    domain_helper::register_domain("vfs", DomainType::VfsDomain(vfs.clone()), true);

    let task = create_domain!(TaskDomainProxy, DomainTypeRaw::TaskDomain, "task")?; // ref to scheduler domain
    domain_helper::register_domain("task", DomainType::TaskDomain(task.clone()), true);

    // we need to register vfs and task domain before init device, because we need to use vfs and task domain in some
    // device init function
    let plic = init_device()?;

    devfs.init()?;
    fatfs.init()?;
    ramfs.init()?;
    procfs.init()?;
    sysfs.init()?;

    // The vfs domain may use the device domain, so we need to init vfs domain after init device domain,
    // also it may use the task domain.
    {
        let initrd = mem::INITRD_DATA.lock();
        let data = initrd.as_ref().unwrap();
        vfs.init(data.as_slice())?;
    }

    task.init()?;

    let syscall = create_domain!(SysCallDomainProxy, DomainTypeRaw::SysCallDomain, "syscall")?;
    syscall.init()?;
    domain_helper::register_domain("syscall", DomainType::SysCallDomain(syscall.clone()), true);

    platform::println!("Load domains done");

    crate::task::register_task_domain(task);
    crate::trap::register_syscall_domain(syscall);
    crate::trap::register_plic_domain(plic);
    platform::println!("Register task domain and syscall domain to trap system");
    Ok(())
}
