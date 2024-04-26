use alloc::sync::Arc;

use constants::{io::Fcntl64Cmd, AlienError, AlienResult};
use interface::InodeID;
use memory_addr::VirtAddr;
use pod::Pod;

use crate::{processor::current_task, vfs_shim::ShimFile};

pub fn do_fcntl(fd: usize, cmd: usize) -> AlienResult<(InodeID, usize)> {
    let cmd = Fcntl64Cmd::try_from(cmd as u32).unwrap();
    let task = current_task().unwrap();
    let file = task.get_file(fd).ok_or(AlienError::EBADF)?;
    match cmd {
        Fcntl64Cmd::F_DUPFD | Fcntl64Cmd::F_DUPFD_CLOEXEC => {
            let fd = task.add_file(file.clone());
            Ok((file.inode_id(), fd))
        }
        _ => Err(AlienError::EINVAL),
    }
}

pub fn do_dup(old_fd: usize, new_fd: Option<usize>) -> AlienResult<isize> {
    let task = current_task().unwrap();
    let file = task.get_file(old_fd).ok_or(AlienError::EBADF)?;
    if new_fd.is_none() {
        let fd = task.add_file(file);
        Ok(fd as isize)
    } else {
        let new_fd = new_fd.unwrap();
        let _file = task.add_file_to_fd(file, new_fd);
        Ok(new_fd as isize)
    }
}
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod)]
struct FdPair {
    fd: [u32; 2],
}
pub fn do_pipe2(r: InodeID, w: InodeID, pipe_ptr: usize) -> AlienResult<isize> {
    let task = current_task().unwrap();
    let r = task.add_file(Arc::new(ShimFile::new(r)));
    let w = task.add_file(Arc::new(ShimFile::new(w)));
    info!("<do_pipe2> r:{},w:{}", r, w);
    let fd_pair = FdPair {
        fd: [r as u32, w as u32],
    };
    task.write_val_to_user(VirtAddr::from(pipe_ptr), &fd_pair)?;
    Ok(0)
}
