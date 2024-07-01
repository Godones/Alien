#![no_std]
#![no_main]

use Mstd::{
    domain::{register_domain, update_domain, DomainTypeRaw},
    fs::{close, open, OpenFlags},
    println,
};

#[no_mangle]
fn main() -> isize {
    let vfs2_fd = open("/tests/gvfs2\0", OpenFlags::O_RDONLY);
    if vfs2_fd < 0 {
        println!("Failed to open /tests/gvfs2");
        return -1;
    } else {
        println!("Opened /tests/gvfs2, fd: {}", vfs2_fd);
        let res = register_domain(vfs2_fd as _, DomainTypeRaw::VfsDomain, "vfs2");
        println!("register domain res: {}", res);

        if res != 0 {
            println!("Failed to register domain vfs2");
            return -1;
        }
        let res = update_domain("vfs", "vfs2", DomainTypeRaw::VfsDomain);
        println!("update domain res: {}", res);
        if res != 0 {
            println!("Failed to update domain vfs");
            return -1;
        }
    }
    close(vfs2_fd as _);
    println!("Test register and update vfs domain");
    0
}
