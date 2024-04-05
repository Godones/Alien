use domain_helper::DomainCreate;
use interface::DomainType;

use crate::domain::*;

pub struct DomainCreateImpl;
impl DomainCreate for DomainCreateImpl {
    fn create_domain(&self, identifier: &str) -> Option<DomainType> {
        match identifier {
            "fatfs" => {
                let fatfs = fatfs_domain();
                domain_helper::register_domain("fatfs", DomainType::FsDomain(fatfs.clone()), false);
                Some(DomainType::FsDomain(fatfs))
            }
            "ramfs" => {
                let ramfs = ramfs_domain();
                domain_helper::register_domain("ramfs", DomainType::FsDomain(ramfs.clone()), false);
                Some(DomainType::FsDomain(ramfs))
            }
            #[cfg(feature = "gui")]
            "virtio-mmio-gpu" => {
                let virtio_mmio_gpu = virtio_mmio_gpu_domain();
                domain_helper::register_domain(
                    "virtio-mmio-gpu",
                    DomainType::GpuDomain(virtio_mmio_gpu.clone()),
                    false,
                );
                Some(DomainType::GpuDomain(virtio_mmio_gpu))
            }
            "virtio-mmio-net" => {
                let virtio_mmio_net = virtio_mmio_net_domain();
                domain_helper::register_domain(
                    "virtio-mmio-net",
                    DomainType::NetDomain(virtio_mmio_net.clone()),
                    false,
                );
                Some(DomainType::NetDomain(virtio_mmio_net))
            }
            "virtio-mmio-block" => {
                let virtio_mmio_block = virtio_mmio_block_domain();
                domain_helper::register_domain(
                    "virtio-mmio-block",
                    DomainType::BlkDeviceDomain(virtio_mmio_block.clone()),
                    false,
                );
                Some(DomainType::BlkDeviceDomain(virtio_mmio_block))
            }
            "virtio-mmio-input" => {
                let virtio_mmio_input = virtio_mmio_input_domain();
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
