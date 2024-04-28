#![no_std]
#![no_main]

use Mstd::{
    domain::{register_domain, update_domain, DomainTypeRaw},
    fs::{open, read, OpenFlags},
    println,
};

#[no_mangle]
fn main() -> isize {
    let sshadow_blk_fd = open("/tests/gsshadow_blk\0", OpenFlags::O_RDONLY);
    if sshadow_blk_fd < 0 {
        println!("Failed to open /tests/gsshadow_blk");
        return -1;
    } else {
        println!("Opened /tests/gsshadow_blk, fd: {}", sshadow_blk_fd);
        let res = register_domain(
            sshadow_blk_fd as _,
            DomainTypeRaw::ShadowBlockDomain,
            "sshadow_blk",
        );
        println!("load_domain res: {}", res);

        if res != 0 {
            println!("Failed to register domain sshadow_blk");
            return -1;
        }
        let res = update_domain(
            "shadow_blk-1",
            "sshadow_blk",
            DomainTypeRaw::ShadowBlockDomain,
        );
        println!("replace_domain res: {}", res);
        if res != 0 {
            println!("Failed to update domain shadow_blk-1");
            return -1;
        }
        let bash_file_test = open("/tests/bash\0", OpenFlags::O_RDWR);
        if bash_file_test < 0 {
            println!("Failed to open /tests/bash");
            return -1;
        } else {
            println!("Opened /tests/bash, fd: {}", bash_file_test);
        }
        let mut buf = [0u8; 100];
        loop {
            let res = read(bash_file_test as usize, &mut buf);
            if res == 0 {
                break;
            }
        }
    }
    println!("Test register and update domain");
    0
}
