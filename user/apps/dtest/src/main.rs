#![no_std]
#![no_main]

use domain_helper::DomainHelperBuilder;
use Mstd::{
    domain::DomainTypeRaw,
    fs::{close, open, read, OpenFlags},
    println,
};

#[no_mangle]
fn main() -> isize {
    let builder = DomainHelperBuilder::new()
        .ty(DomainTypeRaw::ShadowBlockDomain)
        .domain_file_path("/tests/gsshadow_blk\0")
        .domain_file_name("sshadow_blk")
        .domain_name("shadow_blk-1");
    builder.clone().register_domain_file().unwrap();
    builder.update_domain().unwrap();
    let bash_file_test = open("/tests/bash\0", OpenFlags::O_RDWR);
    if bash_file_test < 0 {
        println!("Failed to open /tests/bash");
        return -1;
    }
    let mut buf = [0u8; 100];
    loop {
        let res = read(bash_file_test as usize, &mut buf);
        if res == 0 {
            break;
        }
    }
    close(bash_file_test as _);
    println!("Test register and update shadow_blk domain success");
    0
}
