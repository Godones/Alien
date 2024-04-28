use basic::bus::mmio::VirtioMmioDeviceType;
use constants::{AlienError, AlienResult};
use interface::{DomainType, DomainTypeRaw, GpuDomain};

use crate::{domain_helper, domain_loader::creator::create_gpu_domain, mmio_bus};

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
            let gpu_driver = create_gpu_domain(identifier, None).unwrap();
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
