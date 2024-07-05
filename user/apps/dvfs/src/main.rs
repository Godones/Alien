#![no_std]
#![no_main]

use domain_helper::DomainHelperBuilder;
use Mstd::{domain::DomainTypeRaw, println};

#[no_mangle]
fn main() -> isize {
    let builder = DomainHelperBuilder::new()
        .ty(DomainTypeRaw::VfsDomain)
        .domain_file_path("/tests/gvfs2\0")
        .domain_file_name("vfs2")
        .domain_name("vfs");

    builder.clone().register_domain_file().unwrap();

    builder.update_domain().unwrap();
    println!("Test register and update vfs domain success");
    0
}
