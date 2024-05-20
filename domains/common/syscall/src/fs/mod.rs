mod basic;
mod control;

use alloc::sync::Arc;

use ::basic::{constants::AT_FDCWD, AlienResult};
pub use basic::*;
pub use control::*;
use interface::{InodeID, TaskDomain, VFS_ROOT_ID};
use log::info;

fn user_path_at(
    task_domain: &Arc<dyn TaskDomain>,
    fd: isize,
    path: &str,
) -> AlienResult<(InodeID, InodeID)> {
    info!("user_path_at fd: {}, path:{}", fd, path);
    let res = if !path.starts_with("/") {
        if fd == AT_FDCWD {
            let fs_context = task_domain.fs_info()?;
            (VFS_ROOT_ID, fs_context.1)
        } else {
            let fd = fd as usize;
            let file = task_domain.get_fd(fd)?;
            (VFS_ROOT_ID, file)
        }
    } else {
        (VFS_ROOT_ID, VFS_ROOT_ID)
    };
    Ok(res)
}
