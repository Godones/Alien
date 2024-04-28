#![no_std]
#![no_main]

use Mstd::{
    domain::{register_domain, update_domain, DomainTypeRaw},
    fs::{open, OpenFlags},
    println,
};

#[no_mangle]
fn main() -> isize {
    let mmio_gpu_fd = open("/tests/gvirtio_mmio_gpu\0", OpenFlags::O_RDONLY);
    if mmio_gpu_fd < 0 {
        println!("Failed to open /tests/gvirtio_mmio_gpu");
        return -1;
    } else {
        println!("Opened /tests/gvirtio_mmio_gpu, fd: {}", mmio_gpu_fd);
        let res = register_domain(mmio_gpu_fd as _, DomainTypeRaw::GpuDomain, "virio_mmio_gpu");
        println!("load_domain res: {}", res);

        if res != 0 {
            println!("Failed to register domain virio_mmio_gpu");
            return -1;
        }
        let res = update_domain("", "virio_mmio_gpu", DomainTypeRaw::GpuDomain);
        println!("replace_domain res: {}", res);
        if res != 0 {
            println!("Failed to update domain virio_mmio_gpu");
            return -1;
        }
    }
    println!("Test register and update gpu domain");
    0
}
