mod basic;

use alloc::sync::Arc;

pub use basic::*;
use constants::{io::TeletypeCommand, AlienError, AlienResult};
use interface::{TaskDomain, VfsDomain};
use log::info;

pub fn sys_ioctl(
    vfs: &Arc<dyn VfsDomain>,
    task_domain: &Arc<dyn TaskDomain>,
    fd: usize,
    request: usize,
    argp: usize,
) -> AlienResult<isize> {
    let file = task_domain.get_fd(fd)?;
    let _cmd = TeletypeCommand::try_from(request as u32).map_err(|_| AlienError::EINVAL)?;
    info!(
        "<sys_ioctl> fd:{:?} request:{:?} argp:{:?}",
        fd, request, argp
    );
    vfs.vfs_ioctl(file, request as u32, argp)
        .map(|r| r as isize)
}
