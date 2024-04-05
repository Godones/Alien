use alloc::sync::Arc;

use constants::AlienResult;
use interface::{TaskDomain, VfsDomain};
use rref::RRefVec;

pub fn sys_write(
    vfs: &Arc<dyn VfsDomain>,
    task_domain: &Arc<dyn TaskDomain>,
    fd: usize,
    buf: *const u8,
    len: usize,
) -> AlienResult<isize> {
    let file = task_domain.get_fd(fd)?;
    let tmp_buf = RRefVec::<u8>::new(0, len);
    task_domain.copy_from_user(buf, tmp_buf.as_slice().as_ptr() as *mut u8, len)?;
    let w = vfs.vfs_write(file, &tmp_buf);
    w.map(|x| x as isize)
}

pub fn sys_read(
    vfs: &Arc<dyn VfsDomain>,
    task_domain: &Arc<dyn TaskDomain>,
    fd: usize,
    buf: usize,
    len: usize,
) -> AlienResult<isize> {
    let file = task_domain.get_fd(fd)?;
    let mut tmp_buf = RRefVec::<u8>::new(0, len);
    let r;
    (tmp_buf, r) = vfs.vfs_read(file, tmp_buf)?;
    task_domain.copy_to_user(tmp_buf.as_slice().as_ptr(), buf as _, r)?;
    Ok(r as isize)
}
