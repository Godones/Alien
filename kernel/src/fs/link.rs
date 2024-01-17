use alloc::vec;
use constants::io::{LinkFlags, UnlinkatFlags};
use log::{info, warn};
use syscall_table::syscall_func;

use crate::{fs::user_path_at, task::current_task};
use constants::AlienResult;
/// 一个系统调用，用于创建相对于一个目录某位置处的一个文件的(硬)链接。
///
/// 当传入的 `old_name` 是一个相对地址时，那么 `old_name` 会被解析成基于文件描述符 `old_fd`
/// 所指向的目录地址的一个地址；当传入的 `old_name` 是一个相对地址并且
/// `old_fd` 被特殊的设置为 `AT_FDCWD` 时，`old_name` 会
/// 被解析成基于调用该系统调用的进程当前工作目录的一个地址；
/// 当传入的 `old_name` 是一个绝对地址时，`old_fd` 将被直接忽略。
/// 对于 `new_name` 同理，将根据 'new_fd' 进行解析。
///
/// `flag` 处可以传入的值及其含义包括：
/// + AT_SYMLINK_FOLLOW: 0x400，允许软链接(Follow symbolic links.)。
/// + AT_EMPTY_PATH: 0x1000，允许 old_name 和 new_name 为空字符串(Allow empty relative pathname.).
///
/// `flag` 可以为包括上述值 `OR` 运算的结果。
///
/// 如果成功创建文件链接，`sys_linkat` 将返回0；否则返回-1或错误类型。
/// 出错的情况大致包括：当 `old_name` 或 `new_name` 是一个相对地址，`old_fd` 或 `new_fd` 所指向的文件不是一个目录文件；
/// `old_fd` 或 `new_fd` 不是一个有效的文件描述符等。
///
/// Reference: [link](https://man7.org/linux/man-pages/man2/link.2.html)
#[syscall_func(37)]
pub fn sys_linkat(
    old_fd: isize,
    old_name: *const u8,
    new_fd: isize,
    new_name: *const u8,
    flag: usize,
) -> AlienResult<isize> {
    let flag = LinkFlags::from_bits_truncate(flag as u32);
    let process = current_task().unwrap();
    let old_name = process.transfer_str(old_name);
    let old_path = user_path_at(old_fd, &old_name)?;
    let new_name = process.transfer_str(new_name);
    let new_path = user_path_at(new_fd, &new_name)?;

    warn!(
        "old_path: {},new_path: {}, flag:{:?}",
        old_name, new_name, flag
    );

    let old_dt = old_path.open(None)?;

    new_path.link(old_dt)?;
    Ok(0)
}

/// 一个系统调用，用于删除相对于一个目录某位置处的一个文件的链接。
///
/// `unlinkat`执行的操作将根据`flag`参数是否设置为`AT_REMOVEDIR`而执行`unlink`或`rmdir`操作。
/// 不同情况下，`unlinkat`删除文件链接成功的结果包括：
/// + 如果要删除的链接是软链接，直接删除链接；
/// + 如果要删除的链接是硬连接，并且不是指向该文件的最后一个链接，那么直接删除链接；
/// + 如果要删除的链接是硬连接，并且是指向该文件的最后一个链接
///     + 如果该文件在某进程中处于打开状态，该文件会一直存活到进程关闭其文件描述符，而后被删除。
///     + 如果该文件没有任何进程目前正在打开，该文件会被删除，并回收其所占的存储空间。
///
/// 当传入的`path`是一个相对地址时，那么`path`会被解析成基于文件描述符`fd`
/// 所指向的目录地址的一个地址；当传入的`path`是一个相对地址并且
/// `fd`被特殊的设置为`AT_FDCWD`时，`path`会
/// 被解析成基于调用该系统调用的进程当前工作目录的一个地址；
/// 当传入的`path`是一个绝对地址时，`fd`将被直接忽略。
///
///
/// `flag`处可以传入的值及其含义包括：
/// + AT_REMOVEDIR: 0x200，`sys_linkat`将执行`rmdir`操作。(`rmdir`要求要删除的目录必须为空)
///
/// `flag`可以置为AT_REMOVEDIR或者为0。
///
/// 如果成功删除文件链接，`sys_linkat`将返回0；否则返回-1或错误类型。
///
/// Reference:
/// + [unlink](https://man7.org/linux/man-pages/man2/unlink.2.html)
/// + [rmdir](https://man7.org/linux/man-pages/man2/rmdir.2.html)
///
#[syscall_func(35)]
pub fn sys_unlinkat(fd: isize, path: *const u8, flag: usize) -> AlienResult<isize> {
    let task = current_task().unwrap();
    let path = task.transfer_str(path);
    let flag = UnlinkatFlags::from_bits_truncate(flag as u32);
    info!("unlinkat path: {:?}, flag: {:?}", path, flag);
    let path = user_path_at(fd, &path)?;
    if flag.contains(UnlinkatFlags::AT_REMOVEDIR) {
        path.rmdir()?;
    } else {
        path.unlink()?;
    }
    Ok(0)
}

/// 一个系统调用，用于创建一个指向 `oldname` 的新目录项.
///
/// `old_name` 指明 要链接到的目录位置；新目录项的位置将由 `new_fd` 和 `new_name` 一起解析出，有关解析的相关信息可见 [`user_path_at`]。
///
/// 若创建链接成功，则返回 0；否则返回 -1。
#[syscall_func(36)]
pub fn sys_symlinkat(
    old_name: *const u8,
    new_fd: isize,
    new_name: *const u8,
) -> AlienResult<isize> {
    let process = current_task().unwrap();
    let old_name = process.transfer_str(old_name);
    let new_name = process.transfer_str(new_name);
    let new_path = user_path_at(new_fd, &new_name)?;
    new_path.symlink(&old_name)?;
    Ok(0)
}

/// 一个系统调用，从相对于 一个目录某位置处 的一个软链接文件处读取文件的软链接内容(即链接到文件的路径)。
///
/// 要读取的文件的路径由 `fd` 和 `path` 解析得出。Alien 中有关 `fd` 和 `path` 的路径解析可见 [`user_path_at`] (解析出的文件需是一个软链接文件，否则函数将返回 `ENOENT`)。
/// `buf` 指明了读取内容要保存到的缓冲区首地址，`size` 指明了缓冲区的大小，即所能保存的内容的最大值。
///
/// 若读取成功，则返回读取内容的长度(即链接到文件的路径的长度)；否则返回错误码。
#[syscall_func(78)]
pub fn sys_readlinkat(fd: isize, path: *const u8, buf: *mut u8, size: usize) -> AlienResult<isize> {
    let process = current_task().unwrap();
    let path = process.transfer_str(path);
    info!("readlink path: {}", path);
    let path = user_path_at(fd, &path)?;
    let dt = path.open(None)?;
    let mut empty_buf = vec![0u8; size];
    let r = dt.inode()?.readlink(empty_buf.as_mut_slice())?;
    let buf = process.transfer_buffer(buf, size);
    let mut w = 0;
    for buf in buf {
        let len = buf.len();
        buf.copy_from_slice(&empty_buf[w..w + len]);
        w += len;
        if w >= r {
            break;
        }
    }
    Ok(r as isize)
}
