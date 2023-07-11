use rvfs::dentry::LookUpFlags;
use rvfs::file::{vfs_ioctl, vfs_open_file, FileMode, OpenFlags};
use rvfs::info::VfsTime;
use rvfs::stat::vfs_set_time;

use syscall_define::io::{FaccessatFlags, FaccessatMode, Fcntl64Cmd};
use syscall_define::LinuxErrno;
use syscall_table::syscall_func;

use crate::fs::vfs::VfsProvider;
use crate::fs::{user_path_at, AT_FDCWD};
use crate::task::current_task;
use crate::timer::TimeSpec;

#[syscall_func(25)]
pub fn sys_fcntl(fd: usize, cmd: usize, arg: usize) -> isize {
    let task = current_task().unwrap();
    let file = task.get_file(fd);
    if file.is_none() {
        return LinuxErrno::EBADF as isize;
    }
    let file = file.unwrap();
    let cmd = Fcntl64Cmd::try_from(cmd);
    warn!("fcntl:{:?} {:?} ", cmd, arg);
    match cmd.unwrap() {
        Fcntl64Cmd::F_DUPFD => {
            let fd = task.add_file(file.clone()).unwrap();
            return fd as isize;
        }
        Fcntl64Cmd::F_DUPFD_CLOEXEC => {
            let new_fd = task.add_file(file.clone()).unwrap();
            file.access_inner().flags |= OpenFlags::O_CLOSEEXEC;
            return new_fd as isize;
        }
        Fcntl64Cmd::F_GETFD => {
            return file.access_inner().flags.bits() as isize;
        }
        Fcntl64Cmd::F_SETFD => {
            warn!(
                "fcntl: F_SETFD :{:?}",
                OpenFlags::from_bits_truncate(arg as u32)
            );
            file.access_inner().flags = OpenFlags::from_bits_truncate(arg as u32);
        }
        Fcntl64Cmd::F_GETFL => {
            return file.get_file().access_inner().flags.bits() as isize;
        }
        Fcntl64Cmd::F_SETFL => {
            warn!(
                "fcntl: F_SETFL :{:?}",
                OpenFlags::from_bits_truncate(arg as u32)
            );
            file.get_file().access_inner().flags = OpenFlags::from_bits_truncate(arg as u32);
        }
        _ => {
            return LinuxErrno::EINVAL as isize;
        }
    }
    0
}

// TODO! ioctl
#[syscall_func(29)]
pub fn sys_ioctl(fd: usize, cmd: usize, arg: usize) -> isize {
    let process = current_task().unwrap();
    let file = process.get_file(fd);
    if file.is_none() {
        return LinuxErrno::EBADF as isize;
    }
    let file = file.unwrap();
    let res = vfs_ioctl(file.get_file(), cmd as u32, arg);
    if res.is_err() {
        return -1;
    }
    res.unwrap() as isize
}

#[syscall_func(88)]
pub fn sys_utimensat(fd: usize, path: *const u8, times: *const u8, _flags: usize) -> isize {
    if fd as isize != AT_FDCWD && (fd as isize) < 0 {
        return 0;
    }
    let task = current_task().unwrap();
    let inner = task.access_inner();
    let (atime, mtime) = if times.is_null() {
        (TimeSpec::now(), TimeSpec::now())
    } else {
        let atime_ref = inner.transfer_raw_ptr(times as *mut TimeSpec);
        let mtime_ptr = unsafe { (times as *const TimeSpec).add(1) };
        let mtime_ref = inner.transfer_raw_ptr(mtime_ptr as *mut TimeSpec);
        (*atime_ref, *mtime_ref)
    };
    drop(inner);
    warn!(
        "utimensat: {:?} {:?} {:?} {:?}",
        fd as isize, path, atime, mtime
    );
    if fd as isize > 0 {
        // find in fdmanager
        let file = task.get_file(fd);
        if file.is_none() {
            return LinuxErrno::EBADF as isize;
        }

        let file = file.unwrap();
        let atime = if atime.tv_nsec == 1073741822 {
            // UTIME_OMT
            file.access_inner().atime
        } else if atime.tv_nsec == 1073741823 {
            // UTIME_NOW
            TimeSpec::now()
        } else {
            atime
        };
        let mtime = if mtime.tv_nsec == 1073741822 {
            // UTIME_OMT
            file.access_inner().mtime
        } else if mtime.tv_nsec == 1073741823 {
            TimeSpec::now()
        } else {
            mtime
        };
        warn!("utimensat:  {:?} {:?}", atime, mtime);
        file.access_inner().atime = atime;
        file.access_inner().mtime = mtime;
    } else {
        let path = task.transfer_str(path);
        let path = user_path_at(fd as isize, &path, LookUpFlags::empty()).map_err(|_| -1);
        if path.is_err() {
            return -1;
        }
        warn!("utimensat: {:?}", path);
        let res = vfs_set_time::<VfsProvider>(&path.unwrap(), [VfsTime::default(); 3]);
        if res.is_err() {
            return -1;
        }
    }
    0
}

#[syscall_func(48)]
pub fn faccessat(dirfd: isize, path: usize, mode: usize, flag: usize) -> isize {
    let task = current_task().unwrap();
    let path = task.transfer_str(path as *const u8);
    let path = user_path_at(dirfd, &path, LookUpFlags::empty()).map_err(|_| -1);
    if path.is_err() {
        return -1;
    }
    let path = path.unwrap();
    let mode = FaccessatMode::from_bits_truncate(mode as u32);
    let flag = FaccessatFlags::from_bits_truncate(flag as u32);
    warn!(
        "faccessat file: {:?},flag:{:?}, mode:{:?}",
        path, flag, mode
    );
    let file = vfs_open_file::<VfsProvider>(&path, OpenFlags::O_RDONLY, FileMode::FMODE_RDWR);
    if file.is_err() {
        return -1;
    }
    0
}
