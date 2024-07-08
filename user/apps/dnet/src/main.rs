#![no_std]
#![no_main]

use domain_helper::DomainHelperBuilder;
use Mstd::{domain::DomainTypeRaw, println};

#[no_mangle]
fn main() -> isize {
    let builder = DomainHelperBuilder::new()
        .ty(DomainTypeRaw::NetDeviceDomain)
        .domain_file_path("/tests/gvirtio_mmio_net\0")
        .domain_file_name("virtio_mmio_net_new")
        .domain_name("nic-1");

    builder.clone().register_domain_file().unwrap();
    builder.update_domain().unwrap();

    println!("Test register and update net domain success");
    0
}
