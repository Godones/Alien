pub use domain_info::DomainTypeRaw;

use crate::syscall::{sys_out_mask, sys_register_domain, sys_update_domain};
pub fn register_domain(fd: usize, ty: DomainTypeRaw, domain_file_name: &str) -> isize {
    sys_register_domain(
        fd,
        ty as u8,
        domain_file_name.as_ptr(),
        domain_file_name.len(),
    )
}

pub fn update_domain(domain_name: &str, domain_new_file_name: &str, ty: DomainTypeRaw) -> isize {
    sys_update_domain(
        domain_name.as_ptr(),
        domain_name.len(),
        domain_new_file_name.as_ptr(),
        domain_new_file_name.len(),
        ty as u8,
    )
}

pub fn out_mask() -> isize {
    sys_out_mask()
}
