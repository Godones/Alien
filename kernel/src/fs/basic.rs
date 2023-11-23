use super::im2vim;
use crate::config::AT_FDCWD;
use crate::error::AlienResult;
use crate::fs::file::KernelFile;
use crate::fs::{syscontext_for_vfs, user_path_at, FS, SYSTEM_ROOT_FS};
use crate::task::current_task;
use alloc::sync::Arc;
use alloc::vec;
use core::cmp::min;
use core::ops::Index;
use gmanager::ManagerError;
use pconst::io::{
    FileStat, FsStat, InodeMode, IoVec, MountFlags, OpenFlags, Renameat2Flags, SeekFrom, StatFlags,
};
use pconst::LinuxErrno;
use syscall_table::syscall_func;
use vfscore::path::VfsPath;
use vfscore::utils::{VfsFileStat, VfsFsStat, VfsNodeType, VfsRenameFlag};

/// 用于将一个设备(通常是存储设备)挂载到一个已经存在的目录上，可以挂载文件系统。
#[syscall_func(40)]
pub fn sys_mount(
    source: *const u8,
    dir: *const u8,
    fs_type: *const u8,
    flags: usize,
    data: *const u8,
) -> AlienResult<isize> {
    let process = current_task().unwrap();
    let source = process.transfer_str(source);
    let dir = process.transfer_str(dir);
    let fs_type = process.transfer_str(fs_type);
    assert!(data.is_null());
    let flags = MountFlags::from_bits(flags as u32).unwrap();
    info!(
        "mount special:{:?},dir:{:?},fs_type:{:?},flags:{:?},data:{:?}",
        source, dir, fs_type, flags, data
    );
    let find = FS
        .lock()
        .iter()
        .find(|(name, _)| name.eq(&&fs_type))
        .map(|(_, fs)| fs.clone())
        .ok_or(LinuxErrno::EINVAL)?;
    let path = VfsPath::new(SYSTEM_ROOT_FS.get().unwrap().clone());
    let fs_root = match find.fs_name() {
        name @ ("tmpfs" | "ramfs" | "fat32") => {
            let fs = FS.lock().index(name).clone();
            let dev = if name.eq("fat32") {
                let dev = path.join(source)?.open(None)?;
                Some(dev.inode()?)
            } else {
                None
            };
            let new_fs = fs.i_mount(0, &dir, dev, &[])?;
            new_fs
        }
        _ => return Err(LinuxErrno::EINVAL),
    };
    path.join(dir)?.mount(fs_root, flags.bits())?;
    Ok(0)
}

/// 用于取消一个目录上的文件挂载(卸载一个文件系统)。
#[syscall_func(39)]
pub fn sys_umount(dir: *const u8) -> AlienResult<isize> {
    let process = current_task().unwrap();
    let dir = process.transfer_str(dir);
    info!("umount dir:{:?}", dir);
    let path = VfsPath::new(SYSTEM_ROOT_FS.get().unwrap().clone());
    path.join(dir)?.umount()?;
    Ok(0)
}

#[syscall_func(56)]
pub fn sys_openat(dirfd: isize, path: *const u8, flag: usize, mode: u32) -> AlienResult<isize> {
    if path.is_null() {
        return Err(LinuxErrno::EFAULT);
    }
    let flag = OpenFlags::from_bits_truncate(flag);
    let file_mode = if flag.contains(OpenFlags::O_CREAT) {
        Some(InodeMode::from_bits_truncate(mode))
    } else {
        None
    }
    .map(|x| im2vim(x));
    let process = current_task().unwrap();

    let _path = process.transfer_str(path);
    let path = user_path_at(dirfd, &_path)?;
    warn!(
        "open file: dirfd:[{}], {:?},flag:{:?}, mode:{:?}",
        dirfd, path, flag, file_mode
    );

    let dentry = path.open(file_mode)?;
    let file = KernelFile::new(dentry, flag);

    let fd = process.add_file(Arc::new(file));
    warn!("openat fd: {:?}", fd);
    if fd.is_err() {
        let error = ManagerError::from((fd.unwrap_err()) as usize);
        info!("[vfs] openat error: {:?}", error);
        match error {
            ManagerError::NoSpace => Err(LinuxErrno::EMFILE),
            _ => Err(LinuxErrno::ENOMEM),
        }
    } else {
        Ok(fd.unwrap() as isize)
    }
}

/// 一个系统调用，用于关闭一个文件描述符，以便回收该文件描述符。
///
/// 传入的文件描述符`fd`指向要关闭的文件。如果`fd`所指向的文件已经被`unlink`，
/// 那么在关闭文件描述符后，还将继续执行`unlink`，删除该文件链接，并回收相应的存储空间。
///
/// 如果`sys_close`成功关闭文件描述符，将返回0，否则-1或返回错误的类型。
///
/// Reference: [close](https://man7.org/linux/man-pages/man2/close.2.html)
#[syscall_func(57)]
pub fn sys_close(fd: usize) -> AlienResult<isize> {
    let process = current_task().unwrap();
    let _file = process.remove_file(fd).map_err(|_| LinuxErrno::EBADF)?;
    Ok(0)
}

/// 一个系统调用，用于读取文件的目录项信息。
///
/// 参数：
/// + `fd`: 用于指明操作文件的文件描述符。
/// + `buf`: 用于指明一块缓冲区，保存获取到的目录项信息。
/// + `len`: 用于指明缓冲区的长度。
///
/// 若获取文件的目录项信息成功，则返回获取信息的长度(字节数)；否则返回 -1 表示获取相关信息失败。
///
/// Reference: [sys_getdents](https://man7.org/linux/man-pages/man2/getdents.2.html)
#[syscall_func(61)]
pub fn sys_getdents(fd: usize, buf: *mut u8, len: usize) -> AlienResult<isize> {
    info!("[getdents] fd: {}, buf size: {}", fd, len);
    let process = current_task().unwrap();
    let file = process.get_file(fd).ok_or(LinuxErrno::EBADF)?;
    let user_bufs = process.transfer_buffer(buf, len);
    let mut buf = vec![0u8; len];
    let len = file.readdir(buf.as_mut_slice())?;
    info!("[getdents]: read len: {:?}", len);
    let mut offset = 0;
    // copy dirent_buf to user space
    for user_buf in user_bufs {
        let copy_len = user_buf.len(); // user_bufs len is equal to buf len
        user_buf.copy_from_slice(&buf[offset..offset + copy_len]);
        offset += copy_len;
        if offset >= len {
            break;
        }
    }
    Ok(len as _)
}

/// 一个系统调用，用于将一个文件的大小截断到一个指定长度。与 [`sys_ftruncate`] 功能类似。
///
/// `path` 用于指明要截断文件的路径，`len` 用于指明要截断到的长度。
/// 当文件长度小于 `len` 时，多余的部分填充为'/0'；当文件长度大于 `len` 时，多余的数据将会被直接舍弃。
///
/// 需保证该 `path` 所指出的文件必须是可写的。此外，该调用对于文件的偏移量 offset 将不会改变。
///
/// 当截断成功时，返回 0；否则返回 -1 表示截断出现错误。
/// Reference: https://man7.org/linux/man-pages/man2/truncate64.2.html
#[syscall_func(45)]
pub fn sys_truncate(path: usize, len: usize) -> AlienResult<isize> {
    let process = current_task().unwrap();
    let path = process.transfer_str(path as *const u8);
    let path = user_path_at(AT_FDCWD, &path)?;
    path.truncate(len as u64)?;
    Ok(0)
}

/// 一个系统调用，用于将一个文件的大小截断到一个指定长度。与 [`sys_truncate`] 功能类似。
///
/// `fd` 用于指明要截断文件的文件描述符，`len` 用于指明要截断到的长度。
/// 当文件长度小于 `len` 时，多余的部分填充为'/0'；当文件长度大于 `len` 时，多余的数据将会被直接舍弃。
///
/// 需保证该 `fd` 所指出的文件必须是打开的。此外，该调用对于文件的偏移量 offset 将不会改变。
///
/// 当截断成功时，返回 0；否则返回 -1 表示截断出现错误。
/// Reference: https://man7.org/linux/man-pages/man2/truncate64.2.html
#[syscall_func(46)]
pub fn sys_ftruncate(fd: usize, len: usize) -> AlienResult<isize> {
    let process = current_task().unwrap();
    let file = process.get_file(fd).ok_or(LinuxErrno::EBADF)?;
    file.truncate(len as u64)?;
    Ok(0)
}

/// 一个系统调用，用于从一个打开的文件描述符中读取文件内容。对于每个打开的文件描述符都具有一个偏移量，读取将从该偏移位置开始。
///
/// `fd` 指明了要读取并且已经打开的文件的文件描述符，`buf` 指明了读取内容后所要保存的位置，
/// `len` 指明了缓冲区 `buf` 的大小(即最多读取的内容长度)
///
/// 读取完成后，将返回读取内容的长度(字节数)；如果发生错误，将返回错误类型。
#[syscall_func(63)]
pub fn sys_read(fd: usize, buf: *mut u8, len: usize) -> AlienResult<isize> {
    let process = current_task().unwrap();
    let file = process.get_file(fd).ok_or(LinuxErrno::EBADF)?;
    info!("read file: {:?}, len:{:?}", fd, len);
    let mut buf = process.transfer_buffer(buf, len);

    let mut count = 0;
    for b in buf.iter_mut() {
        let r = file.read(b)?;
        count += r;
        if r != b.len() {
            break;
        }
    }

    Ok(count as _)
}

/// 一个系统调用，用于向一个打开的文件描述符中写入内容。对于每个打开的文件描述符都具有一个偏移量，写入将从该偏移位置开始。
///
/// `fd` 指明了要写入并且已经打开的文件的文件描述符，`buf` 指明了要写入的内容在内存中保存的位置，
/// `len` 指明了缓冲区 `buf` 的大小(即所要写入的内容长度)
///
/// 写入完成后，将返回写入内容的长度(字节数)；如果发生错误，将返回错误类型。
#[syscall_func(64)]
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> AlienResult<isize> {
    let process = current_task().unwrap();
    let file = process.get_file(fd).ok_or(LinuxErrno::EBADF)?;
    let mut buf = process.transfer_buffer(buf, len);
    let mut count = 0;
    for b in buf.iter_mut() {
        let w = file.write(b)?;
        count += w;
        if w != b.len() {
            break;
        }
    }
    Ok(count as _)
}

/// 一个系统调用，用于获取当前工作目录。
/// 获取的工作目录将直接保存在 `buf` 所指向的缓冲区中，`len` 用于指明 `buf` 的长度。
///
/// 获取当前目录成功后，返回 `buf` 的首地址。当 `buf` 为空指针时，会导致函数 panic。
#[syscall_func(17)]
pub fn sys_getcwd(buf: *mut u8, len: usize) -> isize {
    info!("getcwd: {:?}, len: {:?}", buf, len);
    assert!(!buf.is_null());
    let task = current_task().unwrap();
    let cwd = task.access_inner().cwd();

    let mut buf = task.transfer_buffer(buf, len);
    let mut count = 0;
    let path = cwd.cwd.path();
    let mut cwd = path.as_bytes();
    buf.iter_mut().for_each(|buf| {
        // fill buf
        if !cwd.is_empty() {
            let min = min(cwd.len(), buf.len());
            buf[..min].copy_from_slice(&cwd[..min]);
            count += min;
            cwd = &cwd[min..];
        }
    });
    buf[0].as_ptr() as isize
}

/// 一个系统调用，用于切换当前工作目录。`path` 指出要切换到的工作目录。
///
/// 切换工作目录成功后，将返回 0；当输入的 path 不是一个合法的目录项时，会返回 -1 表示切换目录错误。
#[syscall_func(49)]
pub fn sys_chdir(path: *const u8) -> AlienResult<isize> {
    let process = current_task().unwrap();
    let path = process.transfer_str(path);
    let dt = user_path_at(AT_FDCWD, &path)?.open(None)?;

    if dt.inode()?.inode_type() != VfsNodeType::Dir {
        return Err(LinuxErrno::ENOTDIR);
    }
    let fs = dt.inode()?.get_super_block()?.fs_type();
    info!(
        "chdir: {:?} fs: {}, parent:{:?}",
        dt.name(),
        fs.fs_name(),
        dt.parent().is_some()
    );
    process.access_inner().fs_info.cwd = dt;
    Ok(0)
}

/// Like [`sys_chdir`], but uses a file descriptor instead of a path.
#[syscall_func(50)]
pub fn sys_fchdir(fd: usize) -> AlienResult<isize> {
    let process = current_task().unwrap();
    let file = process.get_file(fd).ok_or(LinuxErrno::EBADF)?;
    let dt = file.dentry();
    if dt.inode()?.inode_type() != VfsNodeType::Dir {
        return Err(LinuxErrno::ENOTDIR);
    }
    info!("fchdir: {:?}", dt.path());
    process.access_inner().fs_info.cwd = dt;
    Ok(0)
}

/// 一个系统调用，用于在 相对于一个目录某位置处 路径下创建一个空的目录。功能与 [`sys_mkdir`] 相似。
///
/// 有关对 `dirfd` 和 `mode` 的解析规则以及 flag 的相关设置可见 [`sys_openat`]。成功创建目录则返回 0；否则返回错误码。
///
/// Reference: [mkdirat](https://man7.org/linux/man-pages/man2/mkdirat.2.html)
#[syscall_func(34)]
pub fn sys_mkdirat(dirfd: isize, path: *const u8, mode: u32) -> AlienResult<isize> {
    let process = current_task().unwrap();
    let path = process.transfer_str(path);
    let mut mode = InodeMode::from_bits_truncate(mode);
    warn!("mkdirat path: {}, mode: {:?}", path, mode);
    let path = user_path_at(dirfd, &path)?;
    mode |= InodeMode::DIR;
    assert_eq!(mode & InodeMode::TYPE_MASK, InodeMode::DIR);
    let _ = path.open(Some(im2vim(mode)))?;
    Ok(0)
}

/// 一个系统调用，用于调整一个已经打开的文件描述符的偏移量。文件描述符的偏移量用于确定读写文件时操作的位置。
///
/// 参数：
/// + `fd`: 指明要操作的文件描述符；
/// + `offset`: 指明要调整的偏移量，实际要调整到的位置和 `whence` 的取值有关。
/// + `whence`: 用于定义参数 offset 偏移量对应的参考值，可以取的值及其含义如下：(详情可见 [`SeekFrom`])
///     + `SEEK_SET`: 读写偏移量将指向 offset 字节位置处（从文件头部开始算）；
///     + `SEEK_CUR`: 读写偏移量将指向当前位置偏移量 + offset 字节位置处， offset 可以为正、也可以为负，如果是正数表示往后偏移，如果是负数则表示往前偏移；
///     + `SEEK_END`: 读写偏移量将指向文件末尾 + offset 字节位置处，同样 offset 可以为正、也可以为负，如果是正数表示往后偏移、如果是负数则表示往前偏移。
///
/// 调整成功后，将返回从文件头部开始算起的位置偏移量（字节为单位），也就是当前的读写位置；发生错误将返回错误码，
#[syscall_func(62)]
pub fn sys_lseek(fd: usize, offset: isize, whence: usize) -> AlienResult<isize> {
    let process = current_task().unwrap();
    let file = process.get_file(fd).ok_or(LinuxErrno::EBADF)?;
    let seek = SeekFrom::try_from((whence, offset as usize)).unwrap();
    let r = file.seek(seek).map(|x| x as isize)?;
    Ok(r)
}

/// 一个系统调用，用于向一个打开的文件描述符中写入内容，写入的内容将用一组缓冲区给出。
/// 对于每个打开的文件描述符都具有一个偏移量，写入将从该偏移位置开始。
///
/// `fd` 指明了要执行写入操作并且已经打开的文件的文件描述符，`iovec` 指明了该组缓冲区向量的首地址，
/// `iovcnt` 指明了缓冲区向量的长度，即在这组缓冲区向量包含了多少个缓冲区。(每个缓冲区的大小，通过调用 IoVec::len() 来获取)
///
/// 写入完成后，将返回写入内容的长度(字节数)；如果发生错误，将返回错误类型。
#[syscall_func(66)]
pub fn sys_writev(fd: usize, iovec: usize, iovcnt: usize) -> AlienResult<isize> {
    let process = current_task().unwrap();
    let file = process.get_file(fd).ok_or(LinuxErrno::EBADF)?;
    let mut count = 0;
    for i in 0..iovcnt {
        let mut iov = IoVec::empty();
        let ptr = unsafe { (iovec as *mut IoVec).add(i) };
        process.access_inner().copy_from_user(ptr, &mut iov);
        let base = iov.base;
        if base as usize == 0 {
            // busybox 可能会给stdout两个io_vec，第二个是空地址
            continue;
        }
        let len = iov.len;
        let buf = process.transfer_buffer(base, len);
        for b in buf.iter() {
            let r = file.write(b)?;
            count += r;
        }
    }
    Ok(count as isize)
}

/// 一个系统调用，用于从一个打开的文件描述符中读取文件内容，将读取到的文件内容保存到一组缓冲区中。
/// 对于每个打开的文件描述符都具有一个偏移量，读取将从该偏移位置开始。
///
/// `fd` 指明了要读取并且已经打开的文件的文件描述符，`iovec` 指明了该组缓冲区向量的首地址，
/// `iovcnt` 指明了缓冲区向量的长度，即在这组缓冲区向量包含了多少个缓冲区。(每个缓冲区的大小，通过调用 IoVec::len() 来获取)
///
/// 读取完成后，将返回所有被读取内容的总长度(字节数)。
#[syscall_func(65)]
pub fn sys_readv(fd: usize, iovec: usize, iovcnt: usize) -> AlienResult<isize> {
    let task = current_task().unwrap();
    let file = task.get_file(fd).ok_or(LinuxErrno::EBADF)?;
    let mut count = 0;
    for i in 0..iovcnt {
        let ptr = unsafe { (iovec as *mut IoVec).add(i) };
        let iov = task.transfer_raw_ptr(ptr);
        let base = iov.base;
        if base as usize == 0 || iov.len == 0 {
            continue;
        }
        let len = iov.len;
        let mut buf = task.transfer_buffer(base, len);
        for b in buf.iter_mut() {
            info!("read file: {:?}, len:{:?}", fd, b.len());
            let r = file.read(b)?;
            count += r;
        }
    }
    Ok(count as isize)
}

/// 一个系统调用，用于从一个打开的文件描述符中读取文件内容。与 [`sys_read`] 不同，该调用将指定一个偏移量 `offset`，读取将从该偏移位置开始。
///
/// `fd` 指明了要读取并且已经打开的文件的文件描述符，`buf` 指明了读取内容后所要保存的位置，
/// `len` 指明了缓冲区 `buf` 的大小(即最多读取的内容长度)，`offset` 指明开始读取位置的偏移量。
///
/// 读取完成后，将返回读取内容的长度(字节数)。
#[syscall_func(67)]
pub fn sys_pread(fd: usize, buf: usize, count: usize, offset: u64) -> AlienResult<isize> {
    let task = current_task().unwrap();
    let file = task.get_file(fd).ok_or(LinuxErrno::EBADF)?;
    let mut buf = task.transfer_buffer(buf as *mut u8, count);
    let mut offset = offset;
    let mut count = 0;
    for b in buf.iter_mut() {
        let r = file.read_at(offset, b)?;
        count += r;
        offset += r as u64;
    }
    Ok(count as isize)
}

/// 一个系统调用，用于向一个打开的文件描述符中写入内容。与 [`sys_write`] 不同，该调用将指定一个偏移量 `offset`，写入将从该偏移位置开始。
///
/// `fd` 指明了要写入并且已经打开的文件的文件描述符，`buf` 指明了要写入的内容在内存中保存的位置，
/// `count` 指明了缓冲区 `buf` 的大小(即所要写入的内容长度)
///
/// 写入完成后，将返回写入内容的长度(字节数)；如果发生错误(如输入的 fd 不合法等)，将返回 -1。
#[syscall_func(68)]
pub fn sys_pwrite(fd: usize, buf: usize, count: usize, offset: u64) -> AlienResult<isize> {
    let task = current_task().unwrap();
    let file = task.get_file(fd).ok_or(LinuxErrno::EBADF)?;
    let buf = task.transfer_buffer(buf as *mut u8, count);
    let mut offset = offset;
    let mut count = 0;
    for b in buf.iter() {
        let r = file.write_at(offset, b)?;
        count += r;
        offset += r as u64;
    }

    Ok(count as isize)
}

/// 一个系统调用，用于获取文件的相关信息。功能与 [`sys_fstat`] 类似。
///
/// `sys_fstateat` 中要获取信息的文件 由 `dir_fd` 和 `path` 解析得出。Alien 中有关 `fd` 和 `path` 的路径解析可见 [`user_path_at`]。
/// 获取的信息会保存在 `stat` 指向的 [`FileStat`] 结构中，`flag` 是一组标志位，用于定义相关的操作类型，具体可见 [`StatFlags`]。
///
/// 获取相关信息成功后，函数返回 0；否则函数会返回 -1 表示获取信息出错。
/// 如果输入的 `stat` 为空指针，那么会导致函数 panic。
///
/// Reference: https://man7.org/linux/man-pages/man2/newfstatat.2.html
#[syscall_func(79)]
pub fn sys_fstateat(
    dir_fd: isize,
    path: *const u8,
    stat: *mut u8,
    flag: usize,
) -> AlienResult<isize> {
    let process = current_task().unwrap();
    let path = process.transfer_str(path);
    let flag = StatFlags::from_bits_truncate(flag as u32);
    warn!("sys_fstateat: path: {:?}, flag: {:?}", path, flag);
    let path = user_path_at(dir_fd, &path)?;

    let dt = path.open(None)?;
    let attr = dt.inode()?.get_attr()?;

    let mut file_stat = FileStat::default();
    unsafe {
        (&mut file_stat as *mut FileStat as *mut usize as *mut VfsFileStat).write(attr);
    }
    warn!("sys_fstateat: res: {:?}", file_stat);
    process
        .access_inner()
        .copy_to_user(&file_stat, stat as *mut FileStat);
    Ok(0)
}

/// 一个系统调用，用于获取文件的相关信息。获取的信息会保存在 `stat` 指向的 [`FileStat`] 结构中。
/// `fd` 用于指明要获取信息的文件的文件描述符。
///
/// 获取相关信息成功后，函数返回 0；否则函数会返回 -1 表示获取信息出错。
/// 如果输入的 `stat` 为空指针，那么会导致函数 panic。
#[syscall_func(80)]
pub fn sys_fstat(fd: usize, stat: *mut u8) -> AlienResult<isize> {
    assert!(!stat.is_null());
    let process = current_task().unwrap();
    let file = process.get_file(fd).ok_or(LinuxErrno::EBADF)?;
    let attr = file.get_attr()?;
    let mut file_stat = FileStat::default();
    unsafe {
        (&mut file_stat as *mut FileStat as *mut usize as *mut VfsFileStat).write(attr);
    }
    warn!("sys_fstat: {:?}, res: {:?}", fd, file_stat);
    process
        .access_inner()
        .copy_to_user(&file_stat, stat as *mut FileStat);
    Ok(0)
}

/// 一个系统调用，用于获取一个已挂载的文件系统的使用情况。与 [`sys_statfs`] 的功能类似。
/// 获取到的相关信息将会保存在 `statfs` 所指向的 [`FsStat`] 结构中，`fd` 可以是该已挂载的文件系统下的任意一个文件的文件描述符。
///
/// 如果获取成功，函数会返回 0；否则返回 -1 表示获取信息异常。
/// Reference: https://man7.org/linux/man-pages/man2/fstatfs64.2.html
#[syscall_func(44)]
pub fn sys_fstatfs(fd: isize, buf: *mut u8) -> AlienResult<isize> {
    let process = current_task().unwrap();
    let buf = process.transfer_raw_ptr(buf as *mut FsStat);
    let file = process.get_file(fd as usize).ok_or(LinuxErrno::EBADF)?;
    let fs_stat = file.inode().get_super_block()?.stat_fs()?;
    unsafe {
        (&mut *buf as *mut FsStat as *mut usize as *mut VfsFsStat).write(fs_stat);
    }
    warn!("sys_fstatfs: res: {:#x?}", fs_stat);
    Ok(0)
}

/// 一个系统调用，用于获取一个已挂载的文件系统的使用情况。
/// 获取到的相关信息将会保存在 `statfs` 所指向的 [`FsStat`] 结构中，`path` 可以是该已挂载的文件系统下的任意一个文件的路径。
///
/// 如果获取成功，函数会返回 0；否则返回 -1 表示获取信息异常。
///
/// In libc, [[deprecated]]
#[syscall_func(43)]
pub fn sys_statfs(path: *const u8, statfs: *const u8) -> AlienResult<isize> {
    let process = current_task().unwrap();
    let buf = process.transfer_raw_ptr(statfs as *mut FsStat);
    let path = process.transfer_str(path);

    let path = user_path_at(AT_FDCWD, &path)?;
    let dt = path.open(None)?;
    let fs_stat = dt.inode()?.get_super_block()?.stat_fs()?;

    unsafe {
        (&mut *buf as *mut FsStat as *mut usize as *mut VfsFsStat).write(fs_stat);
    }

    warn!("sys_statfs: [{:?}] res: {:#x?}", path, fs_stat);
    Ok(0)
}

/// Like [`sys_renameat2`].
#[syscall_func(38)]
pub fn sys_renameat(
    old_dirfd: isize,
    old_path: *const u8,
    new_dirfd: isize,
    new_path: *const u8,
) -> AlienResult<isize> {
    let process = current_task().unwrap();
    let old_path = process.transfer_str(old_path);
    let new_path = process.transfer_str(new_path);

    info!(
        "renameat2: {:?} {:?} {:?} {:?}",
        old_dirfd, old_path, new_dirfd, new_path
    );
    let old_path = user_path_at(old_dirfd, &old_path)?;
    let new_path = user_path_at(new_dirfd, &new_path)?;
    old_path.rename_to(
        syscontext_for_vfs(process.access_inner().cwd()),
        new_path,
        VfsRenameFlag::empty(),
    )?;
    Ok(0)
}

/// 一个系统调用，用于更改文件所在的路径名。文件的新/旧路径 将分别使用 new_dirfd/old_dirfd 和 new_path/old_path 解析获得。有关解析的相关设计请查看 [`user_path_at`]。
///
/// 更改文件路径名成功后，函数会返回 0；否则函数返回-1（即当新路径或旧路径中存在不合法的路径 或 在文件系统中修改路径出错时）。
///
/// Reference: https://man7.org/linux/man-pages/man2/renameat.2.html
#[syscall_func(276)]
pub fn sys_renameat2(
    old_dirfd: isize,
    old_path: *const u8,
    new_dirfd: isize,
    new_path: *const u8,
    flag: u32,
) -> AlienResult<isize> {
    let process = current_task().unwrap();
    let old_path = process.transfer_str(old_path);
    let new_path = process.transfer_str(new_path);
    let flag = Renameat2Flags::from_bits_truncate(flag);
    info!(
        "renameat2: {:?} {:?} {:?} {:?}, flag: {:?}",
        old_dirfd, old_path, new_dirfd, new_path, flag
    );
    let old_path = user_path_at(old_dirfd, &old_path)?;
    let new_path = user_path_at(new_dirfd, &new_path)?;

    if flag.contains(Renameat2Flags::RENAME_EXCHANGE)
        && (flag.contains(Renameat2Flags::RENAME_NOREPLACE)
            || flag.contains(Renameat2Flags::RENAME_WHITEOUT))
    {
        return Err(LinuxErrno::EINVAL);
    }

    old_path.rename_to(
        syscontext_for_vfs(process.access_inner().cwd()),
        new_path,
        VfsRenameFlag::from_bits_truncate(flag.bits()),
    )?;
    Ok(0)
}

/// 一个系统调用，用于在文件描述符之间传递数据。
///
/// 从 `in_fd` 读取最多 `count` 个字符，存到 `out_fd` 中。
/// - 如果 `offset != 0`，则其指定了 `in_fd` 中文件的偏移，此时完成后会修改 `offset` 为读取后的位置，但不更新文件内部的 `offset`
/// - 否则，正常更新文件内部的 `offset`
///
/// Reference: [send_file](https://man7.org/linux/man-pages/man2/sendfile64.2.html)
#[syscall_func(71)]
pub fn send_file(
    out_fd: usize,
    in_fd: usize,
    offset_ptr: usize,
    count: usize,
) -> AlienResult<isize> {
    warn!(
        "send_file: in_fd: {:?}, out_fd: {:?}, offset_ptr: {:?}, count: {:?}",
        in_fd, out_fd, offset_ptr, count
    );
    let task = current_task().unwrap();
    let in_file = task.get_file(in_fd).ok_or(LinuxErrno::EINVAL)?;
    let out_file = task.get_file(out_fd).ok_or(LinuxErrno::EINVAL)?;

    if !(in_file.is_readable() && out_file.is_writable()) {
        return Err(LinuxErrno::EBADF);
    }

    let mut buf = vec![0u8; count];

    let nbytes = match offset_ptr {
        // offset 为零则要求更新实际文件
        0 => in_file.read(&mut buf)?,
        _ => {
            // offset 非零则要求不更新实际文件，更新这个用户给的值
            let offset_ptr = task.transfer_raw_ptr(offset_ptr as *mut u64);
            let nbytes = in_file.read_at(*offset_ptr, &mut buf)?;
            *offset_ptr += nbytes as u64;
            nbytes
        }
    };
    info!("sys_sendfile: read {} bytes from in_file", nbytes);
    let w = out_file.write(&buf[0..nbytes as usize])?;
    Ok(w as _)
}

/// 一个系统调用函数，用于包把含更新文件的所有内核缓冲区(包含数据块、指针块、元数据等)都flush到磁盘上。
#[syscall_func(81)]
pub fn sync() -> isize {
    0
}

/// 用于把打开的文件描述符fd相关的所有缓冲元数据和数据都刷新到磁盘上。
#[syscall_func(82)]
pub fn fsync(fd: usize) -> AlienResult<isize> {
    let task = current_task().unwrap();
    let file = task.get_file(fd).ok_or(LinuxErrno::EBADF)?;
    let fs = file.inode().get_super_block()?;
    fs.sync_fs(false)?;
    Ok(0)
}

#[syscall_func(285)]
pub fn copy_file_range(
    fd_in: isize,
    off_in_ptr: usize,
    fd_out: isize,
    off_out_ptr: usize,
    len: usize,
    _flag: usize,
) -> AlienResult<isize> {
    info!(
        "copy_file_range: {:?} {:?} {:?} {:?} {:?}",
        fd_in, off_in_ptr, fd_out, off_out_ptr, len
    );
    let task = current_task().unwrap();
    let in_file = task.get_file(fd_in as usize).ok_or(LinuxErrno::EBADF)?;
    let out_file = task.get_file(fd_out as usize).ok_or(LinuxErrno::EBADF)?;
    if !(in_file.is_readable() && out_file.is_writable()) {
        return Err(LinuxErrno::EBADF);
    }
    let mut buf = vec![0u8; len];
    let r = if off_in_ptr == 0 {
        in_file.read(&mut buf)?
    } else {
        // offset 非零则要求不更新实际文件，更新这个用户给的值
        let off_in_ptr = task.transfer_raw_ptr(off_in_ptr as *mut u64);
        let nr = in_file.read_at(*off_in_ptr, &mut buf)?;
        *off_in_ptr += nr as u64;
        nr
    };
    info!("sys_copy_file_range: read {} bytes from in_file", r);
    let w = if off_out_ptr == 0 {
        out_file.write(&buf[..r])?
    } else {
        let off_out_ptr = task.transfer_raw_ptr(off_out_ptr as *mut u64);
        let wr = out_file.write_at(*off_out_ptr, &buf[..r])?;
        *off_out_ptr += wr as u64;
        wr
    };
    info!("sys_copy_file_range: write {} bytes to out_file", w);
    Ok(w as _)
}
