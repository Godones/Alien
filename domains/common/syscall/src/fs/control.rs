use alloc::sync::Arc;

use basic::{
    constants::io::{Fcntl64Cmd, TeletypeCommand},
    AlienError, AlienResult,
};
use interface::{TaskDomain, VfsDomain};
use log::{debug, info};

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
    let res = vfs.vfs_ioctl(file, request as u32, argp);
    info!("<sys_ioctl> res:{:?}", res);
    res.map(|e| e as isize)
}
pub fn sys_fcntl(
    vfs: &Arc<dyn VfsDomain>,
    task_domain: &Arc<dyn TaskDomain>,
    fd: usize,
    cmd: usize,
    arg: usize,
) -> AlienResult<isize> {
    let raw_cmd = cmd;
    let cmd = Fcntl64Cmd::try_from(cmd as u32).map_err(|_| AlienError::EINVAL)?;
    info!("<sys_fcntl>: {:?} {:?} ", cmd, arg);
    match cmd {
        Fcntl64Cmd::F_DUPFD | Fcntl64Cmd::F_DUPFD_CLOEXEC => {
            let (file, fd) = task_domain.do_fcntl(fd, raw_cmd)?;
            if cmd == Fcntl64Cmd::F_DUPFD_CLOEXEC {
                vfs.do_fcntl(file, raw_cmd, arg)?;
            }
            Ok(fd as isize)
        }
        Fcntl64Cmd::F_GETFD | Fcntl64Cmd::F_SETFD | Fcntl64Cmd::F_GETFL | Fcntl64Cmd::F_SETFL => {
            let file = task_domain.get_fd(fd)?;
            let res = vfs.do_fcntl(file, raw_cmd, arg);
            info!("fcntl:{:?} {:?} res:{:?}", cmd, arg, res);
            res
        }
        Fcntl64Cmd::GETLK | Fcntl64Cmd::SETLK | Fcntl64Cmd::SETLKW => {
            debug!("fcntl: GETLK SETLK SETLKW now ignored");
            Ok(0)
        }
        _ => Err(AlienError::EINVAL),
    }
}

pub fn sys_dup(task_domain: &Arc<dyn TaskDomain>, oldfd: usize) -> AlienResult<isize> {
    task_domain.do_dup(oldfd, None)
}

pub fn sys_dup2(
    task_domain: &Arc<dyn TaskDomain>,
    oldfd: usize,
    newfd: usize,
) -> AlienResult<isize> {
    if oldfd == newfd {
        return Ok(newfd as isize);
    }
    let new_fd = task_domain.do_dup(oldfd, Some(newfd));
    info!("<sys_dup2> oldfd: {:?} newfd: {:?} ", oldfd, new_fd);
    new_fd
}
