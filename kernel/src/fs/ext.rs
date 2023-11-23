use core::cmp::min;
use pconst::LinuxErrno;
use vfscore::path::VfsPath;

use crate::{config::AT_FDCWD, error::AlienResult, fs::user_path_at, task::current_task};

/// 一个系统调用，用于设置文件的 扩展属性(xattrs, Extended Attributes)。
///
/// 扩展属性(xattrs)提供了一个机制用来将一个(键, 值)对永久地关联到文件，让现有的文件系统得以支持在原始设计中未提供的功能。扩展属性是文件系统不可知论者，
/// 应用程序可以通过一个标准的接口来操纵他们，此接口不因文件系统而异。每个扩展属性可以通过唯一的键来区分，键的内容必须是有效的UTF-8，格式为namespace.attribute，
/// 每个键采用完全限定的形式。
///
/// 参数：
/// + `path`: 用于指明要操作文件的路径；
/// + `name`: 用于指明要设置的扩展属性的 `key` 名称，是一个字符串的首地址；
/// + `value`: 用于指明要设置的扩展属性值 `value`，是一段缓冲区的首地址；
/// + `size`: 用于指明缓冲区的长度。请注意该长度最好不要超过一个帧的大小 (4K)；
/// + `flag`: 用于调整操作的类型。目前 Alien 中默认未使用该值，请保证该值为0，否则会导致函数 panic。
///
/// 返回值： 当设置扩展属性成功时，返回 0；否则返回 -1 表示设置失败。
/// Reference: https://man7.org/linux/man-pages/man2/setxattr.2.html
#[syscall_func(5)]
pub fn sys_setxattr(
    path: *const u8,
    name: *const u8,
    value: *const u8,
    size: usize,
    flag: usize,
) -> AlienResult<isize> {
    // we ignore flag
    assert_eq!(flag, 0);
    let process = current_task().unwrap();
    let path = process.transfer_str(path);
    let name = process.transfer_str(name);
    let value = process.transfer_buffer(value, size);
    let path = user_path_at(AT_FDCWD, &path)?;
    path.set_xattr(&name, value[0])?;
    Ok(0)
}

/// 一个系统调用，用于设置文件的 扩展属性(xattrs, Extended Attributes)。在功能上与 [`sys_setxattr`] 相似。
/// 唯一的不同点是 `sys_lsetxattr` 不允许设置软链接文件。
///
/// 目前的实现为直接调用 [`sys_setxattr`]。
#[syscall_func(6)]
pub fn sys_lsetxattr(
    path: *const u8,
    name: *const u8,
    value: *const u8,
    size: usize,
    flag: usize,
) -> AlienResult<isize> {
    sys_setxattr(path, name, value, size, flag)
}

/// 一个系统调用，用于设置文件的 扩展属性(xattrs, Extended Attributes)。在功能和实现上与 [`sys_setxattr`] 相似。
/// 唯一的不同点是 `sys_fsetxattr` 采用文件描述符 `fd` 查找文件，而非文件路径 `path`。
///
/// 有关其它参数和 扩展属性 的相关信息可见 [`sys_setxattr`]。
#[syscall_func(7)]
pub fn sys_fsetxattr(
    fd: usize,
    name: *const u8,
    value: *const u8,
    size: usize,
    flag: usize,
) -> AlienResult<isize> {
    // we ignore flag
    assert_eq!(flag, 0);
    let process = current_task().unwrap();
    let name = process.transfer_str(name);
    let value = process.transfer_buffer(value, size);
    let file = process.get_file(fd).ok_or(LinuxErrno::EBADF)?;
    let path = VfsPath::new(file.dentry());
    path.set_xattr(&name, value[0])?;
    Ok(0)
}

/// 一个系统调用，用于获取文件的扩展属性值。有关 扩展属性 的相关信息可见 [`sys_setxattr`]。
///
/// 参数：
/// + `path`: 用于指明要操作文件的路径；
/// + `name`: 用于指明要获取的扩展属性的 `key` 名称，是一个字符串的首地址；
/// + `value`: 用于指明要获取的扩展属性值 `value` 保存的位置，是一段缓冲区的首地址；
/// + `size`: 用于指明缓冲区的长度。请注意该长度最好不要超过一个帧的大小 (4K)。
///
/// 如果获取扩展属性值成功，返回获取到的扩展属性值的长度；否则返回 -1 表示获取扩展属性值失败。
///
/// Reference: https://man7.org/linux/man-pages/man2/getxattr.2.html
#[syscall_func(8)]
pub fn sys_getxattr(
    path: *const u8,
    name: *const u8,
    value: *const u8,
    size: usize,
) -> AlienResult<isize> {
    let process = current_task().unwrap();
    let path = process.transfer_str(path);
    let name = process.transfer_str(name);
    let mut value = process.transfer_buffer(value, size);
    let path = user_path_at(AT_FDCWD, &path)?;
    let res = path.get_xattr(&name)?;
    let mut copy = 0;
    value.iter_mut().for_each(|x| {
        let min_copy = min(x.len(), res.len() - copy);
        x.copy_from_slice(&res[copy..copy + min_copy]);
        copy += min_copy;
    });
    Ok(copy as _)
}

/// 一个系统调用，用于获取文件的 扩展属性。在功能上与 [`sys_getxattr`] 相似。
/// 唯一的不同点是 `sys_lgetxattr` 不允许获取软链接文件的 扩展属性。
///
/// 目前的实现为直接调用 [`sys_getxattr`]。
#[syscall_func(9)]
pub fn sys_lgetxattr(
    path: *const u8,
    name: *const u8,
    value: *const u8,
    size: usize,
) -> AlienResult<isize> {
    sys_getxattr(path, name, value, size)
}

/// 一个系统调用，用于获取文件的 扩展属性。在功能上与 [`sys_getxattr`] 相似。
/// 唯一的不同点是 `sys_fgetxattr` 采用文件描述符 `fd` 查找文件，而非文件路径 `path`。
///
/// 有关其它参数和 扩展属性 的相关信息可见 [`sys_getxattr`] 和 [`sys_setxattr`]。
#[syscall_func(10)]
pub fn sys_fgetxattr(
    fd: usize,
    name: *const u8,
    value: *const u8,
    size: usize,
) -> AlienResult<isize> {
    let process = current_task().unwrap();
    let name = process.transfer_str(name);
    let mut value = process.transfer_buffer(value, size);
    let file = process.get_file(fd).ok_or(LinuxErrno::EBADF)?;
    let path = VfsPath::new(file.dentry());
    let res = path.get_xattr(&name)?;
    let mut copy = 0;
    value.iter_mut().for_each(|x| {
        let min_copy = min(x.len(), res.len() - copy);
        x.copy_from_slice(&res[copy..copy + min_copy]);
        copy += min_copy;
    });
    Ok(copy as _)
}

/// 一个系统调用，用于获取一个文件的所有扩展属性类型 。有关 扩展属性 的相关信息可见 [`sys_setxattr`]。
///
/// 参数：
/// + `path`: 用于指明要操作文件的路径；
/// + `list`: 用于指明要获取的所有扩展属性类型保存的位置，是一段缓冲区的首地址；
/// + `size`: 用于指明缓冲区的长度。请注意该长度最好不要超过一个帧的大小 (4K)。
///
/// 如果获取扩展属性类型成功，返回获取到的扩展属性类型的长度(总字节数)；否则返回 -1 表示获取扩展属性类型失败。
///
/// Note: 获取到的拓展属性类型类似于 `user.name1/0system.name1/0user.name2/0`，每个拓展属性类型后都会使用 `/0` 表示该种拓展属性类型结束。
///
/// Reference: https://man7.org/linux/man-pages/man2/listxattr.2.html
#[syscall_func(11)]
pub fn sys_listxattr(path: *const u8, list: *const u8, size: usize) -> AlienResult<isize> {
    let process = current_task().unwrap();
    let _path = process.transfer_str(path);
    let _list = process.transfer_buffer(list, size);
    unimplemented!();
}

/// 一个系统调用，用于获取一个文件的所有扩展属性类型。在功能上与 [`sys_listxattr`] 相似。
/// 唯一的不同点是 `sys_llistxattr` 不允许获取软链接文件的所有扩展属性类型。
///
/// 目前的实现为直接调用 [`sys_listxattr`]。
#[syscall_func(12)]
pub fn sys_llistxattr(path: *const u8, list: *const u8, size: usize) -> AlienResult<isize> {
    sys_listxattr(path, list, size)
}

/// 一个系统调用，用于获取一个文件的所有扩展属性类型。在功能上与 [`sys_listxattr`] 相似。
/// 唯一的不同点是 `sys_flistxattr` 采用文件描述符 `fd` 查找文件，而非文件路径 `path`。
///
/// 有关其它参数和 扩展属性 的相关信息可见 [`sys_listxattr`] 和 [`sys_setxattr`]。
#[syscall_func(13)]
pub fn sys_flistxattr(fd: usize, list: *const u8, size: usize) -> AlienResult<isize> {
    let process = current_task().unwrap();
    let _list = process.transfer_buffer(list, size);
    let _file = process.get_file(fd).ok_or(LinuxErrno::EBADF)?;
    unimplemented!();
}

/// 一个系统调用，用于删除文件的某个扩展属性值。有关 扩展属性 的相关信息可见 [`sys_setxattr`]。
///
/// 参数：
/// + `path`: 用于指明要操作文件的路径；
/// + `name`: 用于指明要删除的扩展属性的 `key` 名称，是一个字符串的首地址。
///
/// 如果删除扩展属性值成功，返回0；否则返回 -1 表示删除扩展属性值失败。
///
/// Reference: https://man7.org/linux/man-pages/man2/removexattr.2.html
#[syscall_func(14)]
pub fn sys_removexattr(path: *const u8, name: *const u8) -> AlienResult<isize> {
    let process = current_task().unwrap();
    let _path = process.transfer_str(path);
    let _name = process.transfer_str(name);
    unimplemented!();
}

/// 一个系统调用，用于删除文件的某个扩展属性值。在功能上与 [`sys_removexattr`] 相似。
/// 唯一的不同点是 `sys_lremovexattr` 不允许删除软链接文件的扩展属性值。
///
/// 目前的实现为直接调用 [`sys_removexattr`]。
#[syscall_func(15)]
pub fn sys_lremovexattr(path: *const u8, name: *const u8) -> AlienResult<isize> {
    sys_removexattr(path, name)
}

/// 一个系统调用，用于删除文件的某个扩展属性值型。在功能上与 [`sys_removexattr`] 相似。
/// 唯一的不同点是 `sys_fremovexattr` 采用文件描述符 `fd` 查找文件，而非文件路径 `path`。
///
/// 有关其它参数和 扩展属性 的相关信息可见 [`sys_removexattr`] 和 [`sys_setxattr`]。
#[syscall_func(16)]
pub fn sys_fremovexattr(fd: usize, name: *const u8) -> AlienResult<isize> {
    let process = current_task().unwrap();
    let _name = process.transfer_str(name);
    let _file = process.get_file(fd).ok_or(LinuxErrno::EBADF)?;
    unimplemented!();
}
