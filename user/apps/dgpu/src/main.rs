#![no_std]
#![no_main]

use domain_helper::DomainHelperBuilder;
use Mstd::{domain::DomainTypeRaw, println};

#[no_mangle]
fn main() -> isize {
    let builder = DomainHelperBuilder::new()
        .ty(DomainTypeRaw::GpuDomain)
        .domain_file_path("/tests/gvirtio_mmio_gpu\0")
        .domain_file_name("virtio_mmio_gpu_new")
        .domain_name("virtio_mmio_gpu");
    builder.clone().register_domain_file().unwrap();
    builder.update_domain().unwrap();
    println!("Test register and update gpu domain success");
    0
}
