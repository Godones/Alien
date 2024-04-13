use alloc::{
    boxed::Box,
    collections::BTreeMap,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};

use interface::*;
use ksync::Mutex;
use spin::Lazy;

use crate::{
    domain_helper,
    domain_helper::{alloc_domain_id, DomainCreate},
    domain_loader::loader::DomainLoader,
    domain_proxy::*,
};

static DOMAIN_ELF: Lazy<Mutex<BTreeMap<String, DomainData>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));

#[derive(Clone)]
struct DomainData {
    ty: DomainTypeRaw,
    data: Arc<Vec<u8>>,
}

pub fn register_domain_elf(identifier: &str, elf: Vec<u8>, ty: DomainTypeRaw) {
    let mut binding = DOMAIN_ELF.lock();
    binding.insert(
        identifier.to_string(),
        DomainData {
            ty,
            data: Arc::new(elf),
        },
    );
}

pub fn unregister_domain_elf(identifier: &str) {
    let mut binding = DOMAIN_ELF.lock();
    binding.remove(identifier);
}

pub struct DomainCreateImpl;
impl DomainCreate for DomainCreateImpl {
    fn create_domain(&self, identifier: &str) -> Option<DomainType> {
        match identifier {
            "fatfs" => {
                let fatfs = create_fs_domain("fatfs", None)?;
                fatfs.init().unwrap();
                domain_helper::register_domain("fatfs", DomainType::FsDomain(fatfs.clone()), false);
                Some(DomainType::FsDomain(fatfs))
            }
            "ramfs" => {
                let ramfs = create_fs_domain("ramfs", None)?;
                ramfs.init().unwrap();
                domain_helper::register_domain("ramfs", DomainType::FsDomain(ramfs.clone()), false);
                Some(DomainType::FsDomain(ramfs))
            }
            #[cfg(feature = "gui")]
            "virtio-mmio-gpu" => {
                let virtio_mmio_gpu = create_gpu_domain("virtio-mmio-gpu", None)?;
                domain_helper::register_domain(
                    "virtio-mmio-gpu",
                    DomainType::GpuDomain(virtio_mmio_gpu.clone()),
                    false,
                );
                Some(DomainType::GpuDomain(virtio_mmio_gpu))
            }
            "virtio-mmio-net" => {
                let virtio_mmio_net = create_net_domain("virtio-mmio-net", None)?;
                domain_helper::register_domain(
                    "virtio-mmio-net",
                    DomainType::NetDeviceDomain(virtio_mmio_net.clone()),
                    false,
                );
                Some(DomainType::NetDeviceDomain(virtio_mmio_net))
            }
            "virtio-mmio-block" => {
                let virtio_mmio_block = create_blk_device_domain("virtio-mmio-block", None)?;
                domain_helper::register_domain(
                    "virtio-mmio-block",
                    DomainType::BlkDeviceDomain(virtio_mmio_block.clone()),
                    false,
                );
                Some(DomainType::BlkDeviceDomain(virtio_mmio_block))
            }
            "virtio-mmio-input" => {
                let virtio_mmio_input = create_input_domain("virtio-mmio-input", None)?;
                domain_helper::register_domain(
                    "virtio-mmio-input",
                    DomainType::InputDomain(virtio_mmio_input.clone()),
                    false,
                );
                Some(DomainType::InputDomain(virtio_mmio_input))
            }
            _ => None,
        }
    }
}

pub fn create_devices_domain(
    ident: &str,
    data: Option<Vec<u8>>,
) -> Option<Arc<DevicesDomainProxy>> {
    create_domain(DomainTypeRaw::DevicesDomain, ident, data)
        .map(|(id, domain, _)| Arc::new(DevicesDomainProxy::new(id, domain)))
}
pub fn create_fs_domain(ident: &str, data: Option<Vec<u8>>) -> Option<Arc<FsDomainProxy>> {
    create_domain(DomainTypeRaw::FsDomain, ident, data)
        .map(|(id, domain, _)| Arc::new(FsDomainProxy::new(id, domain)))
}

pub fn create_blk_device_domain(ident: &str, data: Option<Vec<u8>>) -> Option<Arc<BlkDomainProxy>> {
    create_domain(DomainTypeRaw::BlkDeviceDomain, ident, data)
        .map(|(id, domain, loader)| Arc::new(BlkDomainProxy::new(id, domain, loader)))
}

pub fn create_cache_blk_device_domain(
    ident: &str,
    data: Option<Vec<u8>>,
) -> Option<Arc<CacheBlkDomainProxy>> {
    create_domain(DomainTypeRaw::CacheBlkDeviceDomain, ident, data)
        .map(|(id, domain, _)| Arc::new(CacheBlkDomainProxy::new(id, domain)))
}

pub fn create_rtc_domain(ident: &str, data: Option<Vec<u8>>) -> Option<Arc<RtcDomainProxy>> {
    create_domain(DomainTypeRaw::RtcDomain, ident, data)
        .map(|(id, domain, _)| Arc::new(RtcDomainProxy::new(id, domain)))
}

#[cfg(feature = "gui")]
pub fn create_gpu_domain(ident: &str, data: Option<Vec<u8>>) -> Option<Arc<GpuDomainProxy>> {
    create_domain(DomainTypeRaw::GpuDomain, ident, data)
        .map(|(id, domain, _)| Arc::new(GpuDomainProxy::new(id, domain)))
}

pub fn create_input_domain(ident: &str, data: Option<Vec<u8>>) -> Option<Arc<InputDomainProxy>> {
    create_domain(DomainTypeRaw::InputDomain, ident, data)
        .map(|(id, domain, _)| Arc::new(InputDomainProxy::new(id, domain)))
}

pub fn create_uart_domain(ident: &str, data: Option<Vec<u8>>) -> Option<Arc<UartDomainProxy>> {
    create_domain(DomainTypeRaw::UartDomain, ident, data)
        .map(|(id, domain, _)| Arc::new(UartDomainProxy::new(id, domain)))
}

pub fn create_plic_domain(ident: &str, data: Option<Vec<u8>>) -> Option<Arc<PLICDomainProxy>> {
    create_domain(DomainTypeRaw::PLICDomain, ident, data)
        .map(|(id, domain, _)| Arc::new(PLICDomainProxy::new(id, domain)))
}

pub fn create_task_domain(ident: &str, data: Option<Vec<u8>>) -> Option<Arc<TaskDomainProxy>> {
    create_domain(DomainTypeRaw::TaskDomain, ident, data)
        .map(|(id, domain, _)| Arc::new(TaskDomainProxy::new(id, domain)))
}

pub fn create_syscall_domain(
    ident: &str,
    data: Option<Vec<u8>>,
) -> Option<Arc<SysCallDomainProxy>> {
    create_domain(DomainTypeRaw::SysCallDomain, ident, data)
        .map(|(id, domain, _)| Arc::new(SysCallDomainProxy::new(id, domain)))
}

pub fn create_shadow_block_domain(
    ident: &str,
    data: Option<Vec<u8>>,
) -> Option<Arc<ShadowBlockDomainProxy>> {
    create_domain(DomainTypeRaw::ShadowBlockDomain, ident, data)
        .map(|(id, domain, _)| Arc::new(ShadowBlockDomainProxy::new(id, domain)))
}

pub fn create_buf_uart_domain(
    ident: &str,
    data: Option<Vec<u8>>,
) -> Option<Arc<BufUartDomainProxy>> {
    create_domain(DomainTypeRaw::BufUartDomain, ident, data)
        .map(|(id, domain, _)| Arc::new(BufUartDomainProxy::new(id, domain)))
}

pub fn create_net_domain(ident: &str, data: Option<Vec<u8>>) -> Option<Arc<NetDomainProxy>> {
    create_domain(DomainTypeRaw::NetDeviceDomain, ident, data)
        .map(|(id, domain, _)| Arc::new(NetDomainProxy::new(id, domain)))
}

pub fn create_buf_input_domain(
    ident: &str,
    data: Option<Vec<u8>>,
) -> Option<Arc<BufInputDomainProxy>> {
    create_domain(DomainTypeRaw::BufInputDomain, ident, data)
        .map(|(id, domain, _)| Arc::new(BufInputDomainProxy::new(id, domain)))
}

pub fn create_empty_device_domain(
    ident: &str,
    data: Option<Vec<u8>>,
) -> Option<Arc<EmptyDeviceDomainProxy>> {
    create_domain(DomainTypeRaw::EmptyDeviceDomain, ident, data)
        .map(|(id, domain, _)| Arc::new(EmptyDeviceDomainProxy::new(id, domain)))
}

pub fn create_vfs_domain(ident: &str, data: Option<Vec<u8>>) -> Option<Arc<VfsDomainProxy>> {
    create_domain(DomainTypeRaw::VfsDomain, ident, data)
        .map(|(id, domain, _)| Arc::new(VfsDomainProxy::new(id, domain)))
}

pub fn create_devfs_domain(ident: &str, data: Option<Vec<u8>>) -> Option<Arc<DevFsDomainProxy>> {
    create_domain(DomainTypeRaw::DevFsDomain, ident, data)
        .map(|(id, domain, _)| Arc::new(DevFsDomainProxy::new(id, domain)))
}

fn create_domain<T: ?Sized>(
    ty: DomainTypeRaw,
    ident: &str,
    elf: Option<Vec<u8>>,
) -> Option<(u64, Box<T>, DomainLoader)> {
    match elf {
        Some(data) => {
            register_domain_elf(ident, data, ty);
        }
        None => {}
    }
    let data = DOMAIN_ELF.lock().get(ident)?.clone();
    if data.ty != ty {
        return None;
    }
    info!("Load {:?} domain, size: {}KB", ty, data.data.len() / 1024);
    let mut domain_loader = DomainLoader::new(data.data);
    domain_loader.load().unwrap();
    let id = alloc_domain_id();
    let domain = domain_loader.call(id);
    Some((id, domain, domain_loader))
}
