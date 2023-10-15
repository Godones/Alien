use rvfs::dentry::LookUpFlags;
use rvfs::file::{vfs_ioctl, vfs_open_file, FileMode, OpenFlags};
use rvfs::info::VfsTime;
use rvfs::stat::vfs_set_time;

use pconst::io::{FaccessatFlags, FaccessatMode, Fcntl64Cmd, TeletypeCommand};
use pconst::LinuxErrno;
use syscall_table::syscall_func;

use crate::fs::file::{FileIoctlExt, FileSocketExt};
use crate::fs::vfs::VfsProvider;
use crate::fs::{user_path_at, AT_FDCWD};
use crate::task::current_task;
use crate::timer::TimeSpec;

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
/// Reference: [fcntl](https://man7.org/linux/man-pages/man2/fcntl.2.html)
#[syscall_func(25)]
pub fn fcntl(fd: usize, cmd: usize, arg: usize) -> isize {
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
            return if file.access_inner().flags.contains(OpenFlags::O_CLOSEEXEC) {
                1
            } else {
                0
            };
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
            let flag = OpenFlags::from_bits_truncate(arg as u32);
            let real_file = file.get_file();
            real_file.access_inner().flags = flag;
            if real_file.is_socket() {
                let socket = file.get_socketdata();
                if flag.contains(OpenFlags::O_NONBLOCK) {
                    socket.set_socket_nonblock(true);
                } else {
                    socket.set_socket_nonblock(false);
                }
            }
        }
        Fcntl64Cmd::GETLK | Fcntl64Cmd::SETLK | Fcntl64Cmd::SETLKW => {
            warn!("fcntl: GETLK SETLK SETLKW now ignored");
        }
        _ => {
            return LinuxErrno::EINVAL as isize;
        }
    }
    0
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
/// Reference: [ioctl](https://man7.org/linux/man-pages/man2/ioctl.2.html)
#[syscall_func(29)]
pub fn ioctl(fd: usize, cmd: usize, arg: usize) -> isize {
    let process = current_task().unwrap();
    let file = process.get_file(fd);
    if file.is_none() {
        return LinuxErrno::EBADF as isize;
    }
    let file = file.unwrap();
    let cmd = TeletypeCommand::try_from(cmd as u32);
    if cmd.is_err() {
        return LinuxErrno::EBADF.into();
    }
    warn!("ioctl: {:?} {:?} {:?}", fd, cmd, arg);
    let cmd = cmd.unwrap();
    let res = vfs_ioctl(file.get_file(), cmd as u32, arg); // now it is a fake impl
    if res.is_err() {
        return LinuxErrno::ENOTTY.into();
    }
    let res = file.ioctl(cmd as u32, arg);
    res
}

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
///
/// 但目前在 Alien 中，能够正确处理的情况为：
/// + 当用户传入的 fd 为正时，查询的对象规定为文件描述符 fd 对应的文件，即不采用地址解析，然后将相关信息赋值倒 times 处，最终返回 0；
/// + 当传入的 `fd` 为 `AT_FDCWD`时，目前功能还不够完善，但发生错误时，会返回错误码；
/// + 当传入的 `fd` 不合法(既不为正，也不为AT_FDCWD)时，将会导致函数直接返回 0。
#[syscall_func(88)]
pub fn utimensat(fd: usize, path: *const u8, times: *const u8, _flags: usize) -> isize {
    if fd as isize != AT_FDCWD && (fd as isize) < 0 {
        return 0;
    }
    let task = current_task().unwrap();
    let mut inner = task.access_inner();
    let (atime, mtime) = if times.is_null() {
        (TimeSpec::now(), TimeSpec::now())
    } else {
        let mut atime = TimeSpec::new(0, 0);
        let mut mtime = TimeSpec::new(0, 0);
        inner.copy_from_user(times as *const TimeSpec, &mut atime);
        unsafe {
            inner.copy_from_user((times as *const TimeSpec).add(1), &mut mtime);
        }
        (atime, mtime)
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
        // fd == AT_FDCWD or fd == 0
        let path = task.transfer_str(path);
        let path = user_path_at(fd as isize, &path, LookUpFlags::empty()).map_err(|_| -1);
        if path.is_err() {
            return LinuxErrno::EINVAL as isize;
        }
        warn!("utimensat: {:?}", path);
        let res = vfs_set_time::<VfsProvider>(&path.unwrap(), [VfsTime::default(); 3]);
        if res.is_err() {
            error!("utimensat: {:?}", res);
            let res = res.err().unwrap();
            if res.contains("file is not link or dir") {
                return LinuxErrno::ENOTDIR.into();
            }
            return LinuxErrno::ENOENT.into();
        }
    }
    0
}

/// 一个系统调用，用于检测当前进程是否有权限访问一个文件。
///
/// 文件的路径由 `dirfd` 和 `path` 解析得到。解析相关信息可见 [`user_path_at`]。
/// 目前 `mode` 和 `flag` 都未使用。仅检测当前进程是否有打开对应路径下文件的权限。
///
/// 如果有打开的权限，则返回 0；否则返回 -1。
///
#[syscall_func(48)]
pub fn faccessat(dirfd: isize, path: usize, mode: usize, flag: usize) -> isize {
    let task = current_task().unwrap();
    let path = task.transfer_str(path as *const u8);
    let path = user_path_at(dirfd, &path, LookUpFlags::empty()).map_err(|_| -1);
    if path.is_err() {
        return LinuxErrno::ENOENT.into();
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
        return LinuxErrno::ENOENT.into();
    }
    0
}

/// 一个系统调用函数，用于修改文件或目录的权限。(待实现)
///
/// 在Alien系统中，每个文件或目录都有一个权限位，
/// 用于控制该文件或目录的访问权限。sys_chmod函数可以用于修改这些权限位。
///
/// sys_chmod函数需要传入两个参数：第一个参数是需要要修改的文件的文件描述符，
/// 第二个参数是新的权限值。
///
/// Reference: [chmod](https://man7.org/linux/man-pages/man2/chmod.2.html)
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
/// Reference: [chmod](https://man7.org/linux/man-pages/man2/chmod.2.html)
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
