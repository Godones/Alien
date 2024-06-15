#![no_std]
#![no_main]
use Mstd::{
    domain::{register_domain, update_domain, DomainTypeRaw},
    fs::{open, OpenFlags},
    println,
};

#[no_mangle]
fn main() -> isize {
    let mmio_input_fd = open("/tests/gvirtio_mmio_net\0", OpenFlags::O_RDONLY);
    if mmio_input_fd < 0 {
        println!("Failed to open /tests/gvirtio_mmio_net");
        return -1;
    } else {
        println!("Opened /tests/gvirtio_mmio_net, fd: {}", mmio_input_fd);
        let res = register_domain(
            mmio_input_fd as _,
            DomainTypeRaw::NetDeviceDomain,
            "virtio_mmio_net_new",
        );
        println!("register_domain res: {}", res);

        if res != 0 {
            println!("Failed to register domain virtio_mmio_net");
            return -1;
        }
        let res = update_domain(
            "virtio_mmio_net-1",
            "virtio_mmio_net_new",
            DomainTypeRaw::NetDeviceDomain,
        );
        if res != 0 {
            println!("Failed to update domain virtio_mmio_net");
            return -1;
        }
        println!("update_domain virtio_mmio_net ok");
    }
    0
}
