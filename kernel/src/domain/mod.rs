extern crate alloc;

use alloc::sync::Arc;
use domain_helper::{alloc_domain_id, DomainType};
use domain_loader::DomainLoader;
use interface::*;
use log::info;
use proxy::*;

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
static GPU_DOMAIN: &'static [u8] = include_bytes_align_as!(usize, "../../../build/ggpu_domain.bin");
static BLK_DOMAIN: &'static [u8] = include_bytes_align_as!(usize, "../../../build/gblk_domain.bin");
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

fn fatfs_domain() -> Arc<dyn FsDomain> {
    let mut domain = DomainLoader::new(FATFS_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let fatfs = domain.call(id);
    Arc::new(FsDomainProxy::new(id, fatfs))
}

fn uart_domain() -> Arc<dyn UartDomain> {
    let mut domain = DomainLoader::new(UART_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let uart: Arc<dyn UartDomain> = domain.call(id);
    Arc::new(UartDomainProxy::new(id, uart))
}

fn gpu_domain() -> Arc<dyn GpuDomain> {
    let mut domain = DomainLoader::new(GPU_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let gpu: Arc<dyn GpuDomain> = domain.call(id);
    Arc::new(GpuDomainProxy::new(id, gpu))
}

fn blk_domain() -> Arc<dyn BlkDeviceDomain> {
    let mut domain = DomainLoader::new(BLK_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let dev = domain.call::<dyn BlkDeviceDomain>(id);
    info!(
        "dev capacity: {:?}MB",
        dev.get_capacity().unwrap() / 1024 / 1024
    );
    Arc::new(BlkDomainProxy::new(id, dev, domain))
}

fn rtc_domain() -> Arc<dyn RtcDomain> {
    let mut domain = DomainLoader::new(RTC_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let rtc = domain.call(id);
    Arc::new(RtcDomainProxy::new(id, rtc))
}

fn vfs_domain() -> Arc<dyn VfsDomain> {
    let mut domain = DomainLoader::new(VFS_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let vfs = domain.call(id);
    Arc::new(VfsDomainProxy::new(id, vfs))
}

fn cache_blk_domain() -> Arc<dyn CacheBlkDeviceDomain> {
    let mut domain = DomainLoader::new(CACHE_BLK_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let cache_blk = domain.call(id);
    Arc::new(CacheBlkDomainProxy::new(id, cache_blk))
}

fn shadow_blk_domain() -> Arc<dyn BlkDeviceDomain> {
    let mut domain = DomainLoader::new(SHADOW_BLK_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let shadow_blk = domain.call(id);
    Arc::new(BlkDomainProxy::new(id, shadow_blk, domain))
}

fn extern_interrupt_domain() -> Arc<dyn PLICDomain> {
    let mut domain = DomainLoader::new(EXTERN_INTR);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let extern_intr_domain = domain.call(id);
    Arc::new(EIntrDomainProxy::new(id, extern_intr_domain))
}

fn devices_domain() -> Arc<dyn DevicesDomain> {
    let mut domain = DomainLoader::new(DEVICES_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let devices_domain = domain.call(id);
    Arc::new(DevicesDomainProxy::new(id, devices_domain))
}

fn task_domain() -> Arc<dyn TaskDomain> {
    let mut domain = DomainLoader::new(TASK_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let task_domain = domain.call(id);
    Arc::new(TaskDomainProxy::new(id, task_domain))
}

fn syscall_domain() -> Arc<dyn SysCallDomain> {
    let mut domain = DomainLoader::new(SYS_CALL_DOMAIN);
    domain.load().unwrap();
    let id = alloc_domain_id();
    let task_domain = domain.call(id);
    Arc::new(SysCallDomainProxy::new(id, task_domain))
}

pub fn load_domains() {
    info!(
        "Load devices domain, size: {}KB",
        DEVICES_DOMAIN.len() / 1024
    );
    let devices = devices_domain();
    domain_helper::register_domain("devices", DomainType::DevicesDomain(devices));

    info!(
        "Load extern-interrupt domain, size: {}KB",
        EXTERN_INTR.len() / 1024
    );
    let plic = extern_interrupt_domain();
    domain_helper::register_domain("plic", DomainType::PLICDomain(plic));

    info!("Loading uart domain, size: {}KB", UART_DOMAIN.len() / 1024);
    let uart = uart_domain();
    uart.putc('T' as u8).unwrap();
    uart.putc('E' as u8).unwrap();
    uart.putc('S' as u8).unwrap();
    uart.putc('T' as u8).unwrap();
    uart.putc(' ' as u8).unwrap();
    uart.putc('U' as u8).unwrap();
    uart.putc('A' as u8).unwrap();
    uart.putc('R' as u8).unwrap();
    uart.putc('T' as u8).unwrap();
    uart.putc('\n' as u8).unwrap();
    domain_helper::register_domain("uart", DomainType::UartDomain(uart));

    info!("Load blk domain, size: {}KB", BLK_DOMAIN.len() / 1024);
    let dev = blk_domain();
    domain_helper::register_domain("blk", DomainType::BlkDeviceDomain(dev));
    // info!("Load fatfs domain, size: {}KB", FATFS_DOMAIN.len() / 1024);
    // let fs = fatfs_domain();
    // domain_helper::register_domain("fatfs", fs);

    // info!("Loading gpu domain, size: {}KB", GPU_DOMAIN.len() / 1024);
    // let gpu = gpu_domain();
    // domain_helper::register_domain("gpu", DomainType::GpuDomain(gpu));

    info!(
        "Load shadow blk domain, size: {}KB",
        SHADOW_BLK_DOMAIN.len() / 1024
    );
    let shadow_blk = shadow_blk_domain();
    domain_helper::register_domain("shadow_blk", DomainType::BlkDeviceDomain(shadow_blk));

    info!("Load rtc domain, size: {}KB", RTC_DOMAIN.len() / 1024);
    let rtc = rtc_domain();
    domain_helper::register_domain("rtc", DomainType::RtcDomain(rtc));
    info!(
        "Load cache blk domain, size: {}KB",
        CACHE_BLK_DOMAIN.len() / 1024
    );
    let cache_blk = cache_blk_domain();
    domain_helper::register_domain("cache_blk", DomainType::CacheBlkDeviceDomain(cache_blk));
    info!("Load vfs domain, size: {}KB", VFS_DOMAIN.len() / 1024);
    let vfs = vfs_domain();
    domain_helper::register_domain("vfs", DomainType::VfsDomain(vfs));

    info!("Load task domain, size: {}KB", TASK_DOMAIN.len() / 1024);
    let task = task_domain();
    domain_helper::register_domain("task", DomainType::TaskDomain(task.clone()));

    info!(
        "Load syscall domain, size: {}KB",
        SYS_CALL_DOMAIN.len() / 1024
    );
    let syscall = syscall_domain();
    domain_helper::register_domain("syscall", DomainType::SysCallDomain(syscall.clone()));

    platform::println!("Load domains done");

    kcore::register_task_domain(task);
    kcore::register_syscall_domain(syscall);
    platform::println!("Register task domain and syscall domain to trap system done");
}