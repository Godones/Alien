use basic::bus::mmio::VirtioMmioDeviceType;
use interface::{DomainType, DomainTypeRaw, GpuDomain};

use crate::{
    create_domain, domain_helper,
    domain_proxy::GpuDomainProxy,
    error::{AlienError, AlienResult},
    mmio_bus,
};

pub fn update_device_domain(ty: DomainTypeRaw, identifier: &str) -> AlienResult<()> {
    match ty {
        DomainTypeRaw::GpuDomain => {
            let gpu = mmio_bus!()
                .lock()
                .common_devices()
                .iter()
                .find(|d| d.device_type() == VirtioMmioDeviceType::GPU)
                .map(|d| d.clone())
                .unwrap();
            let address = gpu.address().as_usize();
            let size = gpu.io_region().size();
            let gpu_driver =
                create_domain!(GpuDomainProxy, DomainTypeRaw::GpuDomain, identifier).unwrap();
            println!(
                "update gpu domain: {} at address: {:#x} size: {:#x}",
                identifier, address, size
            );
            gpu_driver.init(address..address + size)?;
            domain_helper::register_domain(identifier, DomainType::GpuDomain(gpu_driver), false);
            Ok(())
        }
        _ => Err(AlienError::EINVAL),
    }
}
