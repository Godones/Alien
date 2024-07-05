#![no_std]
#![no_main]

use domain_helper::DomainHelperBuilder;
use Mstd::{domain::DomainTypeRaw, println};

#[no_mangle]
fn main() -> isize {
    let builder = DomainHelperBuilder::new()
        .ty(DomainTypeRaw::InputDomain)
        .domain_file_path("/tests/gvirtio_mmio_input\0")
        .domain_file_name("virtio_mmio_input_new")
        .domain_name("virtio_mmio_input-1");
    builder.clone().register_domain_file().unwrap();
    builder.clone().update_domain().unwrap();
    builder
        .domain_name("virtio_mmio_input-2")
        .update_domain()
        .unwrap();
    println!("Test register and update input domain success");
    0
}
