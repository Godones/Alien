use crate::config::AT_FDCWD;
use crate::fs::user_path_at;
use crate::task::current_task;
use crate::timer::TimeSpec;
use constants::io::{FaccessatFlags, FaccessatMode, Fcntl64Cmd, OpenFlags, TeletypeCommand};
use constants::AlienResult;
use constants::LinuxErrno;
use syscall_table::syscall_func;
use vfscore::utils::*;

const FD_CLOEXEC: usize = 1;

/// 一个系统调用，用于对一个文件提供控制。
///
/// `fd` 指明要操作的文件的描述符；`cmd` 指明控制操作的类型；`arg` 指明操作的参数。
///
/// 目前 Alien 中 fcntl 支持的 `cmd` 类型有：(更多可见 [`Fcntl64Cmd`] )
/// + F_DUPFD: 复制一个现有的文件描述符，此时返回新的文件描述符 new_fd；
/// + F_DUPFD_CLOEXEC: 复制一个现有的文件描述符，同时修改文件的 flags，使得 `O_CLOSEEXEC`位 为1，返回新的文件描述符 new_fd；
/// + F_GETFD: 返回 fd 所指向的文件的 flags 中的 `O_CLOSEEXEC`位。
/// + F_SETFD: 设置 fd 所指向的文件的 flags 的 `O_CLOSEEXEC`位，由参数arg的 `FD_CLOEXEC` 位决定。 设置成功返回 0。
/// + F_GETFL: 返回 fd 所指向的文件的 flags。
/// + F_SETFL: 根据 arg 设置 fd 的 flags，可以采用的 arg 可见 [`OpenFlags`]。
/// + 其它操作类型均会使得函数返回 EINVAL。
///
/// Reference: [fcntl](https:///man7.org/linux/man-pages/man2/fcntl.2.html)
#[syscall_func(25)]
pub fn fcntl(fd: usize, cmd: usize, arg: usize) -> AlienResult<isize> {
    let task = current_task().unwrap();
    let file = task.get_file(fd).ok_or(LinuxErrno::EBADF)?;
    let cmd = Fcntl64Cmd::try_from(cmd).map_err(|_| LinuxErrno::EINVAL)?;
    info!("fcntl:{:?} {:?} ", cmd, arg);
    match cmd {
        Fcntl64Cmd::F_DUPFD => {
            let fd = task.add_file(file.clone()).unwrap();
            return Ok(fd as isize);
        }
        Fcntl64Cmd::F_DUPFD_CLOEXEC => {
            let new_file = file.clone();
            new_file.set_open_flag(new_file.get_open_flag() | OpenFlags::O_CLOEXEC);
            let new_fd = task.add_file(new_file).unwrap();
            return Ok(new_fd as isize);
        }
        Fcntl64Cmd::F_GETFD => {
            return if file.get_open_flag().contains(OpenFlags::O_CLOEXEC) {
                Ok(1)
            } else {
                Ok(0)
            };
        }
        Fcntl64Cmd::F_SETFD => {
            info!("fcntl: F_SETFD :{:?}", arg & FD_CLOEXEC);
            let open_flag = file.get_open_flag();
            if arg & FD_CLOEXEC == 0 {
                file.set_open_flag(open_flag & !OpenFlags::O_CLOEXEC);
            } else {
                file.set_open_flag(open_flag | OpenFlags::O_CLOEXEC);
            }
        }
        Fcntl64Cmd::F_GETFL => {
            return Ok(file.get_open_flag().bits() as isize);
        }
        Fcntl64Cmd::F_SETFL => {
            let flag = OpenFlags::from_bits_truncate(arg);
            info!("fcntl: F_SETFL :{:?}", flag,);
            file.set_open_flag(flag);
        }
        Fcntl64Cmd::GETLK | Fcntl64Cmd::SETLK | Fcntl64Cmd::SETLKW => {
            info!("fcntl: GETLK SETLK SETLKW now ignored");
        }
        _ => {
            return Err(LinuxErrno::EINVAL.into());
        }
    }
    Ok(0)
}

/// 一个系统调用，用于管理 IO 设备。一个字符设备驱动通常会实现设备打开、关闭、读、写等功能，
/// 在一些需要细分的情境下，如果需要扩展新的功能，通常以增设 ioctl() 命令的方式实现。
///
/// `fd` 指明要操作的设备的文件描述符；`cmd` 指明控制操作的类型，
/// 目前 Alien 支持的 ioctl 操作可见 [`TeletypeCommand`] 和 `rvfs` 中有关 `ioctl` 的支持；
/// `arg` 指明操作的参数。
///
/// 根据不同的 ioctl 命令，将有不同的返回值。
///
/// Reference: [ioctl](https:///man7.org/linux/man-pages/man2/ioctl.2.html)
#[syscall_func(29)]
pub fn ioctl(fd: usize, cmd: usize, arg: usize) -> AlienResult<isize> {
    let process = current_task().unwrap();
    let file = process.get_file(fd).ok_or(LinuxErrno::EBADF)?;
    let cmd = TeletypeCommand::try_from(cmd as u32).map_err(|_| LinuxErrno::EINVAL)?;
    info!("ioctl: {:?} {:?} {:?}", fd, cmd, arg);
    let res = file.ioctl(cmd as u32, arg)?;
    Ok(res as isize)
}

const UTIME_NOW: usize = 0x3fffffff;
/// ignore
#[allow(dead_code)]
const UTIME_OMIT: usize = 0x3ffffffe;

/// 一个系统调用，用于获取 相对于一个目录某位置处 的一个文件上一次的访问时间和修改时间。
///
/// 当传入的 `path` 是一个相对地址时，那么 `path` 会被解析成基于文件描述符 `fd` 所指向的目录地址的一个地址；当传入的 `path` 是一个相对地址并且
/// `fd` 被特殊的设置为 `AT_FDCWD` 时，`path` 会被解析成基于调用该系统调用的进程当前工作目录的一个地址；
/// 当传入的 `path` 是一个绝对地址时，`fd`将被直接忽略。
///
/// `times` 指向了一个 `TimeSpec[2]` 的一个数组，其中 TimeSpec[0] 将保存文件上一次访问时间，TimeSpec[1] 将保存文件上一次修改时间。
/// 用户可从传入的两个 `TimeSpec` 结构中修改 `tv_nsec` 字段值来使得查询到的访问时间变为 上一次的操作时间(`UTIME_OMT`)、当前时间(`UTIME_NOW`)或 0。
///
/// `flag` 的值目前还未用到。
#[syscall_func(88)]
pub fn utimensat(
    fd: usize,
    path: *const u8,
    times: *const u8,
    _flags: usize,
) -> AlienResult<isize> {
    if fd as isize != AT_FDCWD && (fd as isize) < 0 {
        return Err(LinuxErrno::EBADF.into());
    }
    let task = current_task().unwrap();

    let dt = if fd as isize > 0 {
        let file = task.get_file(fd).ok_or(LinuxErrno::EBADF)?;
        file.dentry()
    } else {
        let path = task.transfer_str(path);
        let path = user_path_at(fd as isize, &path)?;
        let dt = path.open(None)?;
        dt
    };

    let mut inner = task.access_inner();
    if times.is_null() {
        warn!(
            "utimensat: {:?} {:?} {:?} {:?}",
            fd as isize,
            path,
            TimeSpec::now(),
            TimeSpec::now()
        );
        dt.inode()?.update_time(
            VfsTime::AccessTime(TimeSpec::now().into()),
            TimeSpec::now().into(),
        )?;
        dt.inode()?.update_time(
            VfsTime::AccessTime(TimeSpec::now().into()),
            TimeSpec::now().into(),
        )?;
    } else {
        let mut atime = TimeSpec::new(0, 0);
        let mut mtime = TimeSpec::new(0, 0);
        inner.copy_from_user(times as *const TimeSpec, &mut atime);
        unsafe {
            inner.copy_from_user((times as *const TimeSpec).add(1), &mut mtime);
        }
        warn!(
            "utimensat: {:?} {:?} {:?} {:?}",
            fd as isize, path, atime, mtime
        );
        if atime.tv_nsec == UTIME_NOW {
            dt.inode()?.update_time(
                VfsTime::AccessTime(TimeSpec::now().into()),
                TimeSpec::now().into(),
            )?;
        } else if atime.tv_nsec == UTIME_OMIT {
            // do nothing
        } else {
            dt.inode()?
                .update_time(VfsTime::AccessTime(atime.into()), TimeSpec::now().into())?;
        };
        if mtime.tv_nsec == UTIME_NOW {
            dt.inode()?.update_time(
                VfsTime::ModifiedTime(TimeSpec::now().into()),
                TimeSpec::now().into(),
            )?;
        } else if mtime.tv_nsec == UTIME_OMIT {
            // do nothing
        } else {
            dt.inode()?
                .update_time(VfsTime::ModifiedTime(mtime.into()), TimeSpec::now().into())?;
        };
    };

    Ok(0)
}

/// 一个系统调用，用于检测当前进程是否有权限访问一个文件。
///
/// 文件的路径由 `dirfd` 和 `path` 解析得到。解析相关信息可见 [`user_path_at`]。
/// 目前 `mode` 和 `flag` 都未使用。仅检测当前进程是否有打开对应路径下文件的权限。
///
/// 如果有打开的权限，则返回 0；否则返回 -1。
#[syscall_func(48)]
pub fn faccessat(dirfd: isize, path: usize, mode: usize, flag: usize) -> AlienResult<isize> {
    let task = current_task().unwrap();
    let path = task.transfer_str(path as *const u8);
    let mode = FaccessatMode::from_bits_truncate(mode as u32);
    let flag = FaccessatFlags::from_bits_truncate(flag as u32);
    info!(
        "faccessat file: {:?},flag:{:?}, mode:{:?}",
        path, flag, mode
    );
    // todo! check the AT_SYMLINK_NOFOLLOW flag
    let _path = user_path_at(dirfd, &path)?.open(None)?;
    Ok(0)
}

/// 一个系统调用函数，用于修改文件或目录的权限。(待实现)
///
/// 在Alien系统中，每个文件或目录都有一个权限位，
/// 用于控制该文件或目录的访问权限。sys_chmod函数可以用于修改这些权限位。
///
/// sys_chmod函数需要传入两个参数：第一个参数是需要要修改的文件的文件描述符，
/// 第二个参数是新的权限值。
///
/// Reference: [chmod](https:///man7.org/linux/man-pages/man2/chmod.2.html)
#[syscall_func(52)]
pub fn chmod(_fd: usize, _mode: usize) -> isize {
    0
}

/// (待实现)一个系统调用函数，用于修改相对于某目录某位置处文件或目录的权限。
///
/// 当传入的`path`是一个相对地址时，那么`path`会被解析成基于文件描述符`dirfd`
/// 所指向的目录地址的一个地址；当传入的`path`是一个相对地址并且
/// `dirfd`被特殊的设置为`AT_FDCWD`时，`path`会
/// 被解析成基于调用该系统调用的进程当前工作目录的一个地址；
/// 当传入的`path`是一个绝对地址时，`dirfd`将被直接忽略。
///
/// 当解析出的地址指向的文件是一个软链接时，将根据传入的`flag`的值进行以下的内容：
/// + 若`flag`为0，则将对软链接进行解析，修改软链接的目标文件的权限
/// + 若`flag`为AT_SYMLINK_NOFOLLOW，则将不对软链接进行解析，直接修改该文件的权限
///
/// `flag`处可以传入的值及其含义包括：
/// + AT_SYMLINK_NOFOLLOW: 0x200，如果`path`解析之后指向的文件是一个软链接时，不对软链接进行解析，直接修改该文件的权限
///
/// `flag`可以置为AT_SYMLINK_NOFOLLOW或者为0。
///
/// Reference: [chmod](https:///man7.org/linux/man-pages/man2/chmod.2.html)
#[syscall_func(53)]
pub fn chmodat(_dirfd: usize, _path: usize, _mode: usize, _flags: usize) -> isize {
    0
}

/// 一个系统调用，用于获取并设置当前进程的 `unmask`。在一个进程中，unmask 用于定义新建文件或目录的默认权限。
/// 每次新建一个文件时，文件的默认权限是由 unmask 的值决定的。如果 unmask 值的某位被设置，在新建文件或目录时将禁用对应的权限。
///
/// 函数执行成功后，将会把当前进程的 unmask 值置为传入的 `unmask`，同时返回原来的 unmask 值。
#[syscall_func(55)]
pub fn fchown() -> isize {
    0
}

#[syscall_func(166)]
pub fn unmask(unmask: usize) -> isize {
    let task = current_task().unwrap();
    let old_unmask = task.access_inner().unmask;
    task.access_inner().unmask = unmask;
    old_unmask as isize
}
