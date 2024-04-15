use alloc::sync::Arc;

use constants::AlienResult;
use interface::{DomainTypeRaw, TaskDomain, VfsDomain};
use log::warn;
use rref::{RRef, RRefVec};
use vfscore::utils::VfsFileStat;

pub fn sys_load_domain(
    task_domain: &Arc<dyn TaskDomain>,
    vfs_domain: &Arc<dyn VfsDomain>,
    fd: usize,
    ty: u8,
    domain_name: usize,
    len: usize,
) -> AlienResult<isize> {
    let mut tmp_buf = RRefVec::<u8>::new(0, len);
    task_domain.copy_from_user(domain_name, tmp_buf.as_mut_slice())?;
    let domain_name = core::str::from_utf8(tmp_buf.as_slice()).unwrap();
    let file = task_domain.get_fd(fd)?;
    let attr = RRef::new(VfsFileStat::default());
    let attr = vfs_domain.vfs_getattr(file, attr)?;
    let size = attr.st_size;
    let buf = RRefVec::new(0, size as usize);
    let ty = DomainTypeRaw::try_from(ty).map_err(|_| constants::LinuxErrno::EINVAL)?;
    warn!(
        "<sys_load_domain> domain_name: {:?}, ty:{:?}, size: {}KB",
        domain_name,
        ty,
        size / 1024
    );
    let (buf, read_size) = vfs_domain.vfs_read_at(file, 0, buf)?;
    debug_assert_eq!(read_size, size as usize);
    basic::register_domain(domain_name, ty, buf.as_slice())?;
    Ok(0)
}

pub fn sys_replace_domain(
    task_domain: &Arc<dyn TaskDomain>,
    old_domain_name: usize,
    old_len: usize,
    new_domain_name: usize,
    new_len: usize,
) -> AlienResult<isize> {
    let mut tmp_buf = RRefVec::<u8>::new(0, old_len);
    task_domain.copy_from_user(old_domain_name, tmp_buf.as_mut_slice())?;
    let old_domain_name = core::str::from_utf8(tmp_buf.as_slice()).unwrap();
    let mut tmp_buf = RRefVec::<u8>::new(0, new_len);
    task_domain.copy_from_user(new_domain_name, tmp_buf.as_mut_slice())?;
    let new_domain_name = core::str::from_utf8(tmp_buf.as_slice()).unwrap();
    warn!(
        "<sys_replace_domain> old_domain_name: {:?}, new_domain_name: {:?}",
        old_domain_name, new_domain_name
    );
    basic::replace_domain(old_domain_name, new_domain_name)?;
    Ok(0)
}
