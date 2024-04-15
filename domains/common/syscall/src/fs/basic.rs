use alloc::sync::Arc;

use constants::{AlienError, AlienResult, AT_FDCWD};
use interface::{TaskDomain, VfsDomain};
use log::debug;
use rref::RRefVec;

pub fn sys_openat(
    vfs: &Arc<dyn VfsDomain>,
    task_domain: &Arc<dyn TaskDomain>,
    dirfd: usize,
    path: *const u8,
    flags: usize,
    mode: usize,
) -> AlienResult<isize> {
    if path.is_null() {
        return Err(AlienError::EFAULT);
    }
    let mut tmp_buf = RRefVec::<u8>::new(0, 256);
    let len;
    (tmp_buf, len) = task_domain.read_string_from_user(path as usize, tmp_buf)?;
    let path = core::str::from_utf8(&tmp_buf.as_slice()[..len]).unwrap();
    debug!(
        "<sys_openat> path: {:?} flags: {:?} mode: {:?}",
        path, flags, mode
    );
    let current_root = if dirfd as isize == AT_FDCWD {
        let (_, cwd) = task_domain.fs_info()?;
        cwd
    } else {
        let current_root = task_domain.get_fd(dirfd)?;
        current_root
    };
    let path = RRefVec::from_slice(&path.as_bytes());
    let file = vfs.vfs_open(current_root, &path, mode as _, flags as _)?;
    let fd = task_domain.add_fd(file)?;
    Ok(fd as isize)
}

pub fn sys_write(
    vfs: &Arc<dyn VfsDomain>,
    task_domain: &Arc<dyn TaskDomain>,
    fd: usize,
    buf: *const u8,
    len: usize,
) -> AlienResult<isize> {
    let file = task_domain.get_fd(fd)?;
    let mut tmp_buf = RRefVec::<u8>::new(0, len);
    task_domain.copy_from_user(buf as usize, tmp_buf.as_mut_slice())?;
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
    task_domain.copy_to_user(buf, &tmp_buf.as_slice()[..r])?;
    Ok(r as isize)
}
