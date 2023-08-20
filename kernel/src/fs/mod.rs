use alloc::string::{String, ToString};
use alloc::vec;
use core::cmp::min;

use rvfs::dentry::{vfs_rename, vfs_rmdir, vfs_truncate, vfs_truncate_by_file, LookUpFlags};
use rvfs::file::{
    vfs_close_file, vfs_llseek, vfs_mkdir, vfs_open_file, vfs_read_file, vfs_readdir,
    vfs_write_file, FileMode, FileMode2, OpenFlags, SeekFrom,
};
use rvfs::inode::InodeMode;
use rvfs::link::{vfs_link, vfs_readlink, vfs_symlink, vfs_unlink, LinkFlags};
use rvfs::mount::MountFlags;
use rvfs::path::{vfs_lookup_path, ParsePathType};
use rvfs::stat::{
    vfs_getattr, vfs_getattr_by_file, vfs_getxattr, vfs_getxattr_by_file, vfs_listxattr,
    vfs_listxattr_by_file, vfs_removexattr, vfs_removexattr_by_file, vfs_setxattr,
    vfs_setxattr_by_file, vfs_statfs, vfs_statfs_by_file, KStat, StatFlags,
};
use rvfs::superblock::StatFs;

pub use control::*;
use gmanager::ManagerError;
pub use poll::*;
pub use select::*;
pub use stdio::*;
use syscall_define::io::{FileStat, FsStat, IoVec, UnlinkatFlags};
use syscall_define::LinuxErrno;
use syscall_table::syscall_func;

use crate::fs::file::KFile;
use crate::fs::vfs::VfsProvider;
use crate::task::current_task;

mod stdio;

pub mod basic;
mod control;
pub mod file;
pub mod poll;
pub mod select;
pub mod vfs;

use crate::interrupt::record::{interrupts_info, set_flag, write_interrupt_record};
pub use basic::*;

pub const AT_FDCWD: isize = -100isize;

fn vfs_statfs2fsstat(vfs_res: StatFs) -> FsStat {
    FsStat {
        f_type: vfs_res.fs_type as i64,
        f_bsize: vfs_res.block_size as i64,
        f_blocks: vfs_res.total_blocks,
        f_bfree: vfs_res.free_blocks,
        f_bavail: 0,
        f_files: vfs_res.total_inodes,
        f_ffree: 0,
        f_fsid: [0, 1],
        f_namelen: vfs_res.name_len as isize,
        f_frsize: 0,
        f_flags: 0,
        f_spare: [0; 4],
    }
}

/// (待支持) 一个系统调用，用于将一个设备(通常是存储设备)挂载到一个已经存在的目录上，可以挂载文件系统。
#[syscall_func(40)]
pub fn sys_mount(
    special: *const u8,
    dir: *const u8,
    fs_type: *const u8,
    flags: usize,
    data: *const u8,
) -> isize {
    let process = current_task().unwrap();
    let special = process.transfer_str(special);
    let dir = process.transfer_str(dir);
    let fs_type = process.transfer_str(fs_type);
    let data = process.transfer_str(data);
    assert!(data.is_empty());
    let special = user_path_at(AT_FDCWD, &special, LookUpFlags::empty()).map_err(|_| -1);
    if special.is_err() {
        return -1;
    }
    let special = special.unwrap();
    let dir = user_path_at(AT_FDCWD, &dir, LookUpFlags::empty()).map_err(|_| -1);
    if dir.is_err() {
        return -1;
    }
    let dir = dir.unwrap();

    let flags = MountFlags::from_bits(flags as u32).unwrap();
    warn!(
        "mount special:{:?},dir:{:?},fs_type:{:?},flags:{:?},data:{:?}",
        special, dir, fs_type, flags, data
    );

    // now we return 0 directly
    // todo! rvfs need implement the devfs

    // let ret = do_mount::<VfsProvider>(&special, &dir, &fs_type, flags, None);
    // if ret.is_err() {
    //     return -1;
    // }
    0
}

/// (待支持) 一个系统调用，用于取消一个目录上的文件挂载(卸载一个文件系统)。
#[syscall_func(39)]
pub fn sys_umount(dir: *const u8) -> isize {
    let process = current_task().unwrap();
    let dir = process.transfer_str(dir);
    let dir = user_path_at(AT_FDCWD, &dir, LookUpFlags::empty()).map_err(|_| -1);
    if dir.is_err() {
        return -1;
    }
    let dir = dir.unwrap();
    warn!("umount dir:{:?}", dir);
    // todo! rvfs need implement
    0
}

#[syscall_func(56)]
pub fn sys_openat(dirfd: isize, path: usize, flag: usize, _mode: usize) -> isize {
    // we don't support mode yet
    let file_mode = FileMode2::default();
    let mut file_mode = FileMode::from(file_mode);
    let mut flag = OpenFlags::from_bits(flag as u32).unwrap();
    let process = current_task().unwrap();
    if path == 0 {
        return LinuxErrno::EFAULT.into();
    }
    let path = process.transfer_str(path as *const u8);
    let path = user_path_at(dirfd, &path, LookUpFlags::empty()).map_err(|_| -1);
    if path.is_err() {
        return -1;
    }
    let path = path.unwrap();
    warn!(
        "open file: {:?},flag:{:?}, mode:{:?}",
        path, flag, file_mode
    );

    if flag.contains(OpenFlags::O_BINARY) {
        flag |= OpenFlags::O_RDWR;
    }
    if flag.contains(OpenFlags::O_EXCL) {
        flag -= OpenFlags::O_EXCL;
    }
    file_mode |= FileMode::FMODE_RDWR;
    if path.contains("interrupts") {
        file_mode = FileMode::FMODE_READ;
    }
    let file = vfs_open_file::<VfsProvider>(&path, flag, file_mode);
    if file.is_err() {
        return LinuxErrno::ENOENT.into();
    }
    let file = KFile::new(file.unwrap());
    file.access_inner().path = path;
    let fd = process.add_file(file);
    warn!("openat fd: {:?}", fd);
    if fd.is_err() {
        let error: ManagerError = (fd.unwrap_err() as usize).into();
        error!("[vfs] openat error: {:?}", error);
        match error {
            ManagerError::NoSpace => LinuxErrno::EMFILE.into(),
            _ => LinuxErrno::ENOMEM.into(),
        }
    } else {
        fd.unwrap() as isize
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
pub fn sys_close(fd: usize) -> isize {
    let process = current_task().unwrap();
    let file = process.remove_file(fd);
    if file.is_err() {
        return LinuxErrno::EBADF.into();
    }
    let file = file.unwrap();
    if file.is_unlink() {
        let path = file.unlink_path().unwrap();
        let real_file = file.get_file();
        drop(file);
        let _ = vfs_close_file::<VfsProvider>(real_file);
        let res = vfs_unlink::<VfsProvider>(&path);
        if res.is_err() {
            return -1;
        }
    } else {
        let real_file = file.get_file();
        drop(file);
        let _ = vfs_close_file::<VfsProvider>(real_file);
    }
    0
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
pub fn sys_getdents(fd: usize, buf: *mut u8, len: usize) -> isize {
    let process = current_task().unwrap();
    let file = process.get_file(fd);
    if file.is_none() {
        return -1;
    }
    let file = file.unwrap();
    let user_bufs = process.transfer_buffer(buf, len);
    let mut buf = vec![0u8; len];
    let res = vfs_readdir(file.get_file(), buf.as_mut_slice());
    if res.is_err() {
        return -1;
    }
    let mut offset = 0;
    // copy dirent_buf to user space
    for user_buf in user_bufs {
        let copy_len = user_buf.len(); // user_bufs len is equal to buf len
        user_buf.copy_from_slice(&buf[offset..offset + copy_len]);
        offset += copy_len;
    }
    res.unwrap() as isize
}

/// 一个系统调用，用于将一个文件的大小截断到一个指定长度。与 [`sys_ftruncate`] 功能类似。
///
/// `path` 用于指明要截断文件的路径，`len` 用于指明要截断到的长度。
/// 当文件长度小于 `len` 时，多余的部分填充为'\0'；当文件长度大于 `len` 时，多余的数据将会被直接舍弃。
///
/// 需保证该 `path` 所指出的文件必须是可写的。此外，该调用对于文件的偏移量 offset 将不会改变。
///
/// 当截断成功时，返回 0；否则返回 -1 表示截断出现错误。
/// Reference: https://man7.org/linux/man-pages/man2/truncate64.2.html
#[syscall_func(45)]
pub fn sys_truncate(path: usize, len: usize) -> isize {
    let process = current_task().unwrap();
    let path = process.transfer_str(path as *const u8);
    let res = vfs_truncate::<VfsProvider>(&path, len);
    if res.is_err() {
        return -1;
    }
    0
}

/// 一个系统调用，用于将一个文件的大小截断到一个指定长度。与 [`sys_truncate`] 功能类似。
///
/// `fd` 用于指明要截断文件的文件描述符，`len` 用于指明要截断到的长度。
/// 当文件长度小于 `len` 时，多余的部分填充为'\0'；当文件长度大于 `len` 时，多余的数据将会被直接舍弃。
///
/// 需保证该 `fd` 所指出的文件必须是打开的。此外，该调用对于文件的偏移量 offset 将不会改变。
///
/// 当截断成功时，返回 0；否则返回 -1 表示截断出现错误。
/// Reference: https://man7.org/linux/man-pages/man2/truncate64.2.html
#[syscall_func(46)]
pub fn sys_ftruncate(fd: usize, len: usize) -> isize {
    let process = current_task().unwrap();
    let file = process.get_file(fd);
    if file.is_none() {
        return -1;
    }
    let file = file.unwrap();
    let res = vfs_truncate_by_file(file.get_file(), len);
    if res.is_err() {
        return -1;
    }
    0
}

/// 一个系统调用，用于从一个打开的文件描述符中读取文件内容。对于每个打开的文件描述符都具有一个偏移量，读取将从该偏移位置开始。
///
/// `fd` 指明了要读取并且已经打开的文件的文件描述符，`buf` 指明了读取内容后所要保存的位置，
/// `len` 指明了缓冲区 `buf` 的大小(即最多读取的内容长度)
///
/// 读取完成后，将返回读取内容的长度(字节数)；如果发生错误，将返回错误类型。
#[syscall_func(63)]
pub fn sys_read(fd: usize, buf: *mut u8, len: usize) -> isize {
    let process = current_task().unwrap();
    let file = process.get_file(fd);
    if file.is_none() {
        return LinuxErrno::EBADF.into();
    }
    let file = file.unwrap();
    let mut buf = process.transfer_buffer(buf, len);

    if file.access_inner().path.contains("interrupts") {
        write_interrupt_record(file.get_file(), 1);
    }

    let mut count = 0;
    let mut offset = file.get_file().access_inner().f_pos;
    let mut res = 0;
    for b in buf.iter_mut() {
        let pipe = file.get_file().is_pipe();
        let r = vfs_read_file::<VfsProvider>(file.get_file(), b, offset as u64);
        if r.is_err() {
            if r.err().unwrap() == "Try Again" {
                res = LinuxErrno::EAGAIN.into();
            } else {
                res = LinuxErrno::EIO.into();
            }
            break;
        }
        let r = r.unwrap();
        if file.access_inner().path.contains("interrupts") {
            println!("{} {}", r, offset);
            if r > 0 {
                set_flag(false);
            } else {
                set_flag(true);
            }
        }
        count += r;
        offset += r;
        if pipe && r != 0 {
            break;
        }
        if r != b.len() {
            break;
        }
    }
    if res != 0 {
        return res;
    }
    count as isize
}

/// 一个系统调用，用于向一个打开的文件描述符中写入内容。对于每个打开的文件描述符都具有一个偏移量，写入将从该偏移位置开始。
///
/// `fd` 指明了要写入并且已经打开的文件的文件描述符，`buf` 指明了要写入的内容在内存中保存的位置，
/// `len` 指明了缓冲区 `buf` 的大小(即所要写入的内容长度)
///
/// 写入完成后，将返回写入内容的长度(字节数)；如果发生错误，将返回错误类型。
#[syscall_func(64)]
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    // warn!("sys_write is not implemented yet");
    let process = current_task().unwrap();
    let file = process.get_file(fd);
    if file.is_none() {
        return -1;
    }
    let file = file.unwrap();
    let path = user_path_at(fd as isize, "", LookUpFlags::empty()).map_err(|_| -1);
    if path.is_ok() && path.unwrap().contains("interrupts") {
        return LinuxErrno::EPERM.into();
    }
    let mut buf = process.transfer_buffer(buf, len);
    let mut count = 0;
    let mut offset = file.get_file().access_inner().f_pos;
    let mut res = 0;
    for b in buf.iter_mut() {
        let r = vfs_write_file::<VfsProvider>(file.get_file(), b, offset as u64);
        if r.is_err() {
            if r.err().unwrap().starts_with("pipe_write") {
                error!("pipe_write error: {:?}", r.err().unwrap());
                res = LinuxErrno::EPIPE.into()
            } else if r.err().unwrap() == "Try Again" {
                res = LinuxErrno::EAGAIN.into();
            } else {
                res = LinuxErrno::EIO.into()
            };
            break;
        }
        let r = r.unwrap();
        count += r;
        offset += r;
    }

    if res != 0 {
        return res;
    }
    count as isize
}

/// 一个系统调用，用于获取当前工作目录。
/// 获取的工作目录将直接保存在 `buf` 所指向的缓冲区中，`len` 用于指明 `buf` 的长度。
///
/// 获取当前目录成功后，返回 `buf` 的首地址。当 `buf` 为空指针时，会导致函数 panic。
#[syscall_func(17)]
pub fn sys_getcwd(buf: *mut u8, len: usize) -> isize {
    assert!(!buf.is_null());
    let process = current_task().unwrap();
    let cwd = process.access_inner().cwd();

    let path = vfs_lookup_path(
        cwd.cwd.clone(),
        cwd.cmnt.clone(),
        ParsePathType::Relative("".to_string()),
        LookUpFlags::empty(),
    )
    .unwrap();

    let mut buf = process.transfer_buffer(buf, len);
    let mut count = 0;
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
pub fn sys_chdir(path: *const u8) -> isize {
    let process = current_task().unwrap();
    let path = process.transfer_str(path);
    let file = vfs_open_file::<VfsProvider>(
        path.as_str(),
        OpenFlags::O_RDWR | OpenFlags::O_DIRECTORY,
        FileMode::FMODE_RDWR,
    );
    if file.is_err() {
        return -1;
    }
    let file = file.unwrap();

    if file.f_dentry.access_inner().d_inode.mode != InodeMode::S_DIR {
        return -1;
    }
    process.access_inner().fs_info.cwd = file.f_dentry.clone();
    process.access_inner().fs_info.cmnt = file.f_mnt.clone();
    0
}

/// 一个系统调用，用于在 path 路径下创建一个空的目录。创建成功返回 0；否则返回 -1。
#[syscall_func(83)]
pub fn sys_mkdir(path: *const u8) -> isize {
    info!("sys_mkdir");
    let process = current_task().unwrap();
    let path = process.transfer_str(path);
    let file = vfs_mkdir::<VfsProvider>(&path, FileMode::FMODE_WRITE);
    if file.is_err() {
        return -1;
    }
    0
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
pub fn sys_lseek(fd: usize, offset: isize, whence: usize) -> isize {
    let process = current_task().unwrap();
    let file = process.get_file(fd);
    if file.is_none() {
        return -1;
    }
    let file = file.unwrap();
    let seek = SeekFrom::from((whence, offset as usize));
    let res = vfs_llseek(file.get_file(), seek);
    warn!("sys_lseek: {:?}, res: {:?}", seek, res);
    if res.is_err() {
        if file.get_file().is_pipe() {
            return LinuxErrno::ESPIPE.into();
        }
        return LinuxErrno::EINVAL.into();
    }
    res.unwrap() as isize
}

/// 一个系统调用，用于获取文件的相关信息。获取的信息会保存在 `stat` 指向的 [`FileStat`] 结构中。
/// `fd` 用于指明要获取信息的文件的文件描述符。
///
/// 获取相关信息成功后，函数返回 0；否则函数会返回 -1 表示获取信息出错。
/// 如果输入的 `stat` 为空指针，那么会导致函数 panic。
#[syscall_func(80)]
pub fn sys_fstat(fd: usize, stat: *mut u8) -> isize {
    assert!(!stat.is_null());
    let process = current_task().unwrap();
    let file = process.get_file(fd);
    if file.is_none() {
        return -1;
    }
    let file = file.unwrap();
    // let stat = process.transfer_raw_ptr(stat);
    let attr = vfs_getattr_by_file(file.get_file());
    if attr.is_err() {
        return -1;
    }
    let mut attr = attr.unwrap();
    attr.st_atime_sec = file.access_inner().atime.tv_sec as u64;
    attr.st_atime_nsec = file.access_inner().atime.tv_nsec as u64;
    attr.st_mtime_sec = file.access_inner().mtime.tv_sec as u64;
    attr.st_mtime_nsec = file.access_inner().mtime.tv_nsec as u64;

    let mut file_stat = FileStat::default();
    unsafe {
        (&mut file_stat as *mut FileStat as *mut usize as *mut KStat).write(attr);
    }
    file_stat.st_mode |= 0o755;
    if file_stat.st_ino == 0 {
        file_stat.st_ino = 999;
    }
    warn!("sys_fstat: {:?}, res: {:?}", fd, file_stat);
    process
        .access_inner()
        .copy_to_user(&file_stat, stat as *mut FileStat);
    0
}

/// 一个系统调用，用于创建相对于一个目录某位置处的一个文件的(硬)链接。
///
/// 当传入的 `old_name` 是一个相对地址时，那么 `old_name` 会被解析成基于文件描述符 `old_fd`
/// 所指向的目录地址的一个地址；当传入的 `old_name` 是一个相对地址并且
/// `old_fd` 被特殊的设置为 `AT_FDCWD` 时，`old_name` 会
/// 被解析成基于调用该系统调用的进程当前工作目录的一个地址；
/// 当传入的 `old_name` 是一个绝对地址时，`old_fd` 将被直接忽略。
/// 对于 `new_name` 同理，将根据 'new_fd' 进行解析。
///
/// 在 `Alien` 使用的 `rvfs` 中，对一个文件路径 `path` 是相对路径还是绝对路径的的判断条件如下：
/// + 绝对路径：以 `\` 开头，如 `\file1.txt`，表示根目录下的 `file1.txt` 文件；
/// + 相对路径: 以 `.\` 或者 `..\` 或者其它开头，如 `.\file1.txt `，表示 `dirfd` 所指向的目录下的 `file1.txt` 文件。
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
) -> isize {
    let flag = LinkFlags::from_bits(flag as u32);
    if flag.is_none() {
        return -1;
    }
    let flag = flag.unwrap();
    let flag = flag - LinkFlags::AT_SYMLINK_FOLLOW - LinkFlags::AT_EMPTY_PATH;
    if !flag.is_empty() {
        warn!("sys_linkat: flag is not empty");
        return -1;
    }
    // we try to find the old path according to the old_fd and old_name and flag
    let mut lookup_flag = LookUpFlags::empty();
    if flag.contains(LinkFlags::AT_SYMLINK_FOLLOW) {
        lookup_flag |= LookUpFlags::READ_LINK;
    }
    if flag.contains(LinkFlags::AT_EMPTY_PATH) {
        lookup_flag |= LookUpFlags::EMPTY;
    }
    let process = current_task().unwrap();
    let old_name = process.transfer_str(old_name);
    let old_path = user_path_at(old_fd, &old_name, lookup_flag).map_err(|_| -1);
    if old_path.is_err() {
        return -1;
    }
    let new_name = process.transfer_str(new_name);
    let new_path = user_path_at(new_fd, &new_name, lookup_flag).map_err(|_| -1);
    if new_path.is_err() {
        return -1;
    }
    let old_path = old_path.unwrap();
    let new_path = new_path.unwrap();
    warn!("old_path: {},new_path: {}", old_path, new_path);
    let res = vfs_link::<VfsProvider>(old_path.as_str(), new_path.as_str());
    if res.is_err() {
        return -1;
    }
    0
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
/// 在`Alien`使用的`rvfs`中，对一个文件路径`path`是相对路径还是绝对路径的的判断条件如下：
/// + 绝对路径：以`\`开头，如`\file1.txt`，表示根目录下的`file1.txt`文件；
/// + 相对路径: 以`.\`或者`..\`或者其它开头，如`.\file1.txt`，表示`dirfd`所指向的目录下的`file1.txt`文件。
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
pub fn sys_unlinkat(fd: isize, path: *const u8, flag: usize) -> isize {
    let task = current_task().unwrap();
    let path = task.transfer_str(path);
    let path = user_path_at(fd, &path, LookUpFlags::empty()).map_err(|_| -1);
    if path.is_err() {
        return -1;
    }
    // TODO we need make sure the file of the path is not being used
    let path = path.unwrap();
    if path.contains("interrupts") {
        return LinuxErrno::EPERM.into();
    }
    // find the file, checkout whether it is being used
    let file = vfs_open_file::<VfsProvider>(&path, OpenFlags::empty(), FileMode::FMODE_RDWR);
    if file.is_err() {
        return -1;
    }
    let is_used = task.file_existed(file.unwrap());
    warn!("sys_unlinkat: is_used: {}", is_used.is_some());
    if is_used.is_some() {
        let file = is_used.unwrap();
        file.set_unlink(path);
    } else {
        let flag = UnlinkatFlags::from_bits_truncate(flag as u32);
        let res = if flag.contains(UnlinkatFlags::AT_REMOVEDIR) {
            vfs_rmdir::<VfsProvider>(path.as_str())
        } else {
            vfs_unlink::<VfsProvider>(path.as_str())
        };
        if res.is_err() {
            error!(
                "sys_unlinkat: vfs_unlink {} failed, flag:{:?}, {}",
                path,
                flag,
                res.err().unwrap()
            );
            return -1;
        }
    }
    0
}

/// 一个系统调用，用于创建一个指向 `oldname` 的新目录项.
///
/// `old_name` 指明 要链接到的目录位置；新目录项的位置将由 `new_fd` 和 `new_name` 一起解析出，有关解析的相关信息可见 [`user_path_at`]。
///
/// 若创建链接成功，则返回 0；否则返回 -1。
#[syscall_func(36)]
pub fn sys_symlinkat(old_name: *const u8, new_fd: isize, new_name: *const u8) -> isize {
    let process = current_task().unwrap();
    let old_name = process.transfer_str(old_name);
    let new_name = process.transfer_str(new_name);
    let new_path = user_path_at(new_fd, &new_name, LookUpFlags::empty()).map_err(|_| -1);
    if new_path.is_err() {
        return -1;
    }
    let new_path = new_path.unwrap();
    let res = vfs_symlink::<VfsProvider>(old_name.as_str(), new_path.as_str());
    if res.is_err() {
        return -1;
    }
    0
}

/// 一个系统调用，从相对于 一个目录某位置处 的一个软链接文件处读取文件的软链接内容(即链接到文件的路径)。
///
/// 要读取的文件的路径由 `fd` 和 `path` 解析得出。Alien 中有关 `fd` 和 `path` 的路径解析可见 [`user_path_at`] (解析出的文件需是一个软链接文件，否则函数将返回 `ENOENT`)。
/// `buf` 指明了读取内容要保存到的缓冲区首地址，`size` 指明了缓冲区的大小，即所能保存的内容的最大值。
///
/// 若读取成功，则返回读取内容的长度(即链接到文件的路径的长度)；否则返回错误码。
#[syscall_func(78)]
pub fn sys_readlinkat(fd: isize, path: *const u8, buf: *mut u8, size: usize) -> isize {
    let process = current_task().unwrap();
    let path = process.transfer_str(path);
    let path = user_path_at(fd, &path, LookUpFlags::empty()).map_err(|_| -1);
    if path.is_err() {
        return -1;
    }
    let path = path.unwrap();
    let mut buf = process.transfer_buffer(buf, size);

    warn!("readlink path: {}", path);
    let res = vfs_readlink::<VfsProvider>(path.as_str(), buf[0]);
    if res.is_err() {
        return LinuxErrno::ENOENT.into();
    }
    let res = res.unwrap();
    res as isize
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
pub fn sys_fstateat(dir_fd: isize, path: *const u8, stat: *mut u8, flag: usize) -> isize {
    let process = current_task().unwrap();
    let path = process.transfer_str(path);
    let path = user_path_at(dir_fd, &path, LookUpFlags::empty()).map_err(|_| -1);
    if path.is_err() {
        return -1;
    }
    let path = path.unwrap();
    let flag = StatFlags::from_bits(flag as u32);
    if flag.is_none() {
        return -1;
    }
    let flag = flag.unwrap();
    warn!("sys_fstateat: path: {}, flag: {:?}", path, flag);
    let res = vfs_getattr::<VfsProvider>(path.as_str(), flag);
    if res.is_err() {
        return LinuxErrno::ENOENT as isize;
    }
    let res = res.unwrap();
    let mut file_stat = FileStat::default();
    unsafe {
        (&mut file_stat as *mut FileStat as *mut usize as *mut KStat).write(res);
    }
    file_stat.st_mode |= 0o755;
    if file_stat.st_ino == 0 {
        file_stat.st_ino = 999;
    }
    warn!("sys_fstateat: res: {:?}", file_stat);
    process
        .access_inner()
        .copy_to_user(&file_stat, stat as *mut FileStat);
    0
}

/// 一个系统调用，用于获取一个已挂载的文件系统的使用情况。与 [`sys_statfs`] 的功能类似。
/// 获取到的相关信息将会保存在 `statfs` 所指向的 [`FsStat`] 结构中，`fd` 可以是该已挂载的文件系统下的任意一个文件的文件描述符。
///
/// 如果获取成功，函数会返回 0；否则返回 -1 表示获取信息异常。
/// Reference: https://man7.org/linux/man-pages/man2/fstatfs64.2.html
#[syscall_func(44)]
pub fn sys_fstatfs(fd: isize, buf: *mut u8) -> isize {
    let process = current_task().unwrap();
    let buf = buf as *mut FsStat;
    let buf = process.transfer_raw_ptr(buf);
    let file = process.get_file(fd as usize);
    if file.is_none() {
        return -1;
    }
    let file = file.unwrap();
    let res = vfs_statfs_by_file(file.get_file());
    if res.is_err() {
        return -1;
    }
    let res = res.unwrap();
    *buf = vfs_statfs2fsstat(res);
    0
}

/// 一个系统调用，用于获取一个已挂载的文件系统的使用情况。
/// 获取到的相关信息将会保存在 `statfs` 所指向的 [`FsStat`] 结构中，`path` 可以是该已挂载的文件系统下的任意一个文件的路径。
///
/// 如果获取成功，函数会返回 0；否则返回 -1 表示获取信息异常。
#[syscall_func(43)]
pub fn sys_statfs(path: *const u8, statfs: *const u8) -> isize {
    let process = current_task().unwrap();
    let buf = statfs as *mut FsStat;
    let buf = process.transfer_raw_ptr(buf);
    let path = process.transfer_str(path);
    let res = vfs_statfs::<VfsProvider>(path.as_str());
    trace!("sys_statfs: res: {:#x?}", res);
    if res.is_err() {
        return -1;
    }
    let res = res.unwrap();
    *buf = vfs_statfs2fsstat(res);
    0
}

/// 一个系统调用，用于更改文件所在的路径名。文件的新\旧路径 将分别使用 new_dirfd\old_dirfd 和 new_path\old_path 解析获得。有关解析的相关设计请查看 [`user_path_at`]。
///
/// 更改文件路径名成功后，函数会返回 0；否则函数返回-1（即当新路径或旧路径中存在不合法的路径 或 在文件系统中修改路径出错时）。
///
/// Reference: https://man7.org/linux/man-pages/man2/renameat.2.html
#[syscall_func(276)]
pub fn sys_renameat(
    old_dirfd: isize,
    old_path: *const u8,
    new_dirfd: isize,
    new_path: *const u8,
) -> isize {
    let process = current_task().unwrap();
    let old_path = process.transfer_str(old_path);
    let new_path = process.transfer_str(new_path);
    let old_path = user_path_at(old_dirfd, &old_path, LookUpFlags::empty()).map_err(|_| -1);
    if old_path.is_err() {
        return -1;
    }
    let old_path = old_path.unwrap();
    if old_path.contains("interrupts") {
        return LinuxErrno::EPERM.into();
    }
    let new_path = user_path_at(new_dirfd, &new_path, LookUpFlags::empty()).map_err(|_| -1);
    if new_path.is_err() {
        return -1;
    }
    let new_path = new_path.unwrap();
    let res = vfs_rename::<VfsProvider>(old_path.as_str(), new_path.as_str());
    if res.is_err() {
        return -1;
    }
    0
}

/// 一个系统调用，用于在 相对于一个目录某位置处 路径下创建一个空的目录。功能与 [`sys_mkdir`] 相似。
///
/// 有关对 `dirfd` 和 `path` 的解析规则以及 flag 的相关设置可见 [`sys_openat`]。成功创建目录则返回 0；否则返回错误码。
///
/// Reference: [mkdirat](https://man7.org/linux/man-pages/man2/mkdirat.2.html)
#[syscall_func(34)]
pub fn sys_mkdirat(dirfd: isize, path: *const u8, flag: usize) -> isize {
    let process = current_task().unwrap();
    let path = process.transfer_str(path);
    let path = user_path_at(dirfd, &path, LookUpFlags::empty()).map_err(|_| -1);
    if path.is_err() {
        return -1;
    }
    let path = path.unwrap();
    let flag = OpenFlags::from_bits_truncate(flag as u32);
    warn!("mkdirat path: {}, flag: {:?}", path, flag);
    let mut mode = FileMode::FMODE_READ;
    if flag.contains(OpenFlags::O_WRONLY) {
        mode |= FileMode::FMODE_WRITE;
    }
    let res = vfs_mkdir::<VfsProvider>(path.as_str(), mode);
    if res.is_err() {
        error!("mkdirat failed: {:?}", res);
        return LinuxErrno::EEXIST.into();
    }
    0
}

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
) -> isize {
    // we ignore flag
    assert_eq!(flag, 0);
    let process = current_task().unwrap();
    let path = process.transfer_str(path);
    let name = process.transfer_str(name);
    let value = process.transfer_buffer(value, size);
    let res = vfs_setxattr::<VfsProvider>(path.as_str(), name.as_str(), value[0]);
    if res.is_err() {
        return -1;
    }
    0
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
) -> isize {
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
) -> isize {
    // we ignore flag
    assert_eq!(flag, 0);
    let process = current_task().unwrap();
    let name = process.transfer_str(name);
    let value = process.transfer_buffer(value, size);
    let file = process.get_file(fd);
    if file.is_none() {
        return -1;
    }
    let file = file.unwrap();
    let res = vfs_setxattr_by_file(file.get_file(), name.as_str(), value[0]);
    if res.is_err() {
        return -1;
    }
    0
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
pub fn sys_getxattr(path: *const u8, name: *const u8, value: *const u8, size: usize) -> isize {
    let process = current_task().unwrap();
    let path = process.transfer_str(path);
    let name = process.transfer_str(name);
    let mut value = process.transfer_buffer(value, size);
    // assert_eq!(value.len(),1);
    if value.is_empty() {
        value.push(&mut [0u8; 0])
    }
    let res = vfs_getxattr::<VfsProvider>(path.as_str(), name.as_str(), value[0]);
    if res.is_err() {
        return -1;
    }
    let res = res.unwrap();
    res as isize
}

/// 一个系统调用，用于获取文件的 扩展属性。在功能上与 [`sys_getxattr`] 相似。
/// 唯一的不同点是 `sys_lgetxattr` 不允许获取软链接文件的 扩展属性。
///
/// 目前的实现为直接调用 [`sys_getxattr`]。
#[syscall_func(9)]
pub fn sys_lgetxattr(path: *const u8, name: *const u8, value: *const u8, size: usize) -> isize {
    sys_getxattr(path, name, value, size)
}

/// 一个系统调用，用于获取文件的 扩展属性。在功能上与 [`sys_getxattr`] 相似。
/// 唯一的不同点是 `sys_fgetxattr` 采用文件描述符 `fd` 查找文件，而非文件路径 `path`。
///
/// 有关其它参数和 扩展属性 的相关信息可见 [`sys_getxattr`] 和 [`sys_setxattr`]。
#[syscall_func(10)]
pub fn sys_fgetxattr(fd: usize, name: *const u8, value: *const u8, size: usize) -> isize {
    let process = current_task().unwrap();
    let name = process.transfer_str(name);
    let mut value = process.transfer_buffer(value, size);
    // assert_eq!(value.len(),1);
    if value.is_empty() {
        value.push(&mut [0u8; 0])
    }
    let file = process.get_file(fd);
    if file.is_none() {
        return -1;
    }
    let file = file.unwrap();
    let res = vfs_getxattr_by_file(file.get_file(), name.as_str(), value[0]);
    if res.is_err() {
        return -1;
    }
    let res = res.unwrap();
    res as isize
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
/// Note: 获取到的拓展属性类型类似于 `user.name1\0system.name1\0user.name2\0`，每个拓展属性类型后都会使用 `\0` 表示该种拓展属性类型结束。
///
/// Reference: https://man7.org/linux/man-pages/man2/listxattr.2.html
#[syscall_func(11)]
pub fn sys_listxattr(path: *const u8, list: *const u8, size: usize) -> isize {
    let process = current_task().unwrap();
    let path = process.transfer_str(path);
    let mut list = process.transfer_buffer(list, size);
    if list.is_empty() {
        list.push(&mut [0u8; 0])
    }
    let res = vfs_listxattr::<VfsProvider>(path.as_str(), list[0]);
    if res.is_err() {
        return -1;
    }
    let res = res.unwrap();
    res as isize
}

/// 一个系统调用，用于获取一个文件的所有扩展属性类型。在功能上与 [`sys_listxattr`] 相似。
/// 唯一的不同点是 `sys_llistxattr` 不允许获取软链接文件的所有扩展属性类型。
///
/// 目前的实现为直接调用 [`sys_listxattr`]。
#[syscall_func(12)]
pub fn sys_llistxattr(path: *const u8, list: *const u8, size: usize) -> isize {
    sys_listxattr(path, list, size)
}

/// 一个系统调用，用于获取一个文件的所有扩展属性类型。在功能上与 [`sys_listxattr`] 相似。
/// 唯一的不同点是 `sys_flistxattr` 采用文件描述符 `fd` 查找文件，而非文件路径 `path`。
///
/// 有关其它参数和 扩展属性 的相关信息可见 [`sys_listxattr`] 和 [`sys_setxattr`]。
#[syscall_func(13)]
pub fn sys_flistxattr(fd: usize, list: *const u8, size: usize) -> isize {
    let process = current_task().unwrap();
    let mut list = process.transfer_buffer(list, size);
    if list.is_empty() {
        list.push(&mut [0u8; 0])
    }
    let file = process.get_file(fd);
    if file.is_none() {
        return -1;
    }
    let file = file.unwrap();
    let res = vfs_listxattr_by_file(file.get_file(), list[0]);
    if res.is_err() {
        return -1;
    }
    let res = res.unwrap();
    res as isize
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
pub fn sys_removexattr(path: *const u8, name: *const u8) -> isize {
    let process = current_task().unwrap();
    let path = process.transfer_str(path);
    let name = process.transfer_str(name);
    let res = vfs_removexattr::<VfsProvider>(path.as_str(), name.as_str());
    if res.is_err() {
        return -1;
    }
    0
}

/// 一个系统调用，用于删除文件的某个扩展属性值。在功能上与 [`sys_removexattr`] 相似。
/// 唯一的不同点是 `sys_lremovexattr` 不允许删除软链接文件的扩展属性值。
///
/// 目前的实现为直接调用 [`sys_removexattr`]。
#[syscall_func(15)]
pub fn sys_lremovexattr(path: *const u8, name: *const u8) -> isize {
    sys_removexattr(path, name)
}

/// 一个系统调用，用于向一个打开的文件描述符中写入内容，写入的内容将用一组缓冲区给出。
/// 对于每个打开的文件描述符都具有一个偏移量，写入将从该偏移位置开始。
///
/// `fd` 指明了要执行写入操作并且已经打开的文件的文件描述符，`iovec` 指明了该组缓冲区向量的首地址，
/// `iovcnt` 指明了缓冲区向量的长度，即在这组缓冲区向量包含了多少个缓冲区。(每个缓冲区的大小，通过调用 IoVec::len() 来获取)
///
/// 写入完成后，将返回写入内容的长度(字节数)；如果发生错误，将返回错误类型。
#[syscall_func(66)]
pub fn sys_writev(fd: usize, iovec: usize, iovcnt: usize) -> isize {
    let process = current_task().unwrap();
    let file = process.get_file(fd);
    if file.is_none() {
        return LinuxErrno::EBADF.into();
    }
    let file = file.unwrap();
    let mut count = 0;
    let mut res = 0;
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

        let mut offset = file.get_file().access_inner().f_pos;

        for b in buf.iter() {
            let r = vfs_write_file::<VfsProvider>(file.get_file(), b, offset as u64);
            if r.is_err() {
                if r.err().unwrap().starts_with("pipe_write") {
                    error!("pipe_write error: {:?}", r.err().unwrap());
                    res = LinuxErrno::EPIPE.into()
                } else {
                    res = LinuxErrno::EIO.into()
                };
                break;
            }
            let r = r.unwrap();
            count += r;
            offset += r;
        }
        if res != 0 {
            break;
        }
    }

    if res != 0 {
        return res;
    }
    count as isize
}

/// 一个系统调用，用于从一个打开的文件描述符中读取文件内容，将读取到的文件内容保存到一组缓冲区中。
/// 对于每个打开的文件描述符都具有一个偏移量，读取将从该偏移位置开始。
///
/// `fd` 指明了要读取并且已经打开的文件的文件描述符，`iovec` 指明了该组缓冲区向量的首地址，
/// `iovcnt` 指明了缓冲区向量的长度，即在这组缓冲区向量包含了多少个缓冲区。(每个缓冲区的大小，通过调用 IoVec::len() 来获取)
///
/// 读取完成后，将返回所有被读取内容的总长度(字节数)。
#[syscall_func(65)]
pub fn sys_readv(fd: usize, iovec: usize, iovcnt: usize) -> isize {
    let task = current_task().unwrap();
    let file = task.get_file(fd);
    if file.is_none() {
        return -1;
    }
    let file = file.unwrap();
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

        let mut offset = file.get_file().access_inner().f_pos;
        buf.iter_mut().for_each(|b| {
            warn!(
                "read file: {:?}, offset:{:?}, len:{:?}",
                fd,
                offset,
                b.len()
            );
            let r = vfs_read_file::<VfsProvider>(file.get_file(), b, offset as u64).unwrap();
            count += r;
            offset += r;
        });
    }
    count as isize
}

/// 一个系统调用，用于从一个打开的文件描述符中读取文件内容。与 [`sys_read`] 不同，该调用将指定一个偏移量 `offset`，读取将从该偏移位置开始。
///
/// `fd` 指明了要读取并且已经打开的文件的文件描述符，`buf` 指明了读取内容后所要保存的位置，
/// `len` 指明了缓冲区 `buf` 的大小(即最多读取的内容长度)，`offset` 指明开始读取位置的偏移量。
///
/// 读取完成后，将返回读取内容的长度(字节数)。
#[syscall_func(67)]
pub fn sys_pread(fd: usize, buf: usize, count: usize, offset: usize) -> isize {
    let task = current_task().unwrap();
    let file = task.get_file(fd);
    if file.is_none() {
        return -1;
    }
    let file = file.unwrap();
    let mut buf = task.transfer_buffer(buf as *mut u8, count);
    let mut offset = offset;
    let old_file_offset = file.get_file().access_inner().f_pos;
    let mut count = 0;
    buf.iter_mut().for_each(|b| {
        let r = vfs_read_file::<VfsProvider>(file.get_file(), b, offset as u64).unwrap();
        count += r;
        offset += r;
    });
    file.get_file().access_inner().f_pos = old_file_offset;
    count as isize
}

/// 一个系统调用，用于向一个打开的文件描述符中写入内容。与 [`sys_write`] 不同，该调用将指定一个偏移量 `offset`，写入将从该偏移位置开始。
///
/// `fd` 指明了要写入并且已经打开的文件的文件描述符，`buf` 指明了要写入的内容在内存中保存的位置，
/// `count` 指明了缓冲区 `buf` 的大小(即所要写入的内容长度)
///
/// 写入完成后，将返回写入内容的长度(字节数)；如果发生错误(如输入的 fd 不合法等)，将返回 -1。
#[syscall_func(68)]
pub fn sys_pwrite(fd: usize, buf: usize, count: usize, offset: usize) -> isize {
    let task = current_task().unwrap();
    let file = task.get_file(fd);
    if file.is_none() {
        return -1;
    }
    let file = file.unwrap();
    let buf = task.transfer_buffer(buf as *mut u8, count);
    let mut offset = offset;
    let old_file_offset = file.get_file().access_inner().f_pos;
    let mut count = 0;
    buf.iter().for_each(|b| {
        let r = vfs_write_file::<VfsProvider>(file.get_file(), b, offset as u64).unwrap();
        count += r;
        offset += r;
    });
    file.get_file().access_inner().f_pos = old_file_offset;
    count as isize
}

/// 一个系统调用，用于删除文件的某个扩展属性值型。在功能上与 [`sys_removexattr`] 相似。
/// 唯一的不同点是 `sys_fremovexattr` 采用文件描述符 `fd` 查找文件，而非文件路径 `path`。
///
/// 有关其它参数和 扩展属性 的相关信息可见 [`sys_removexattr`] 和 [`sys_setxattr`]。
#[syscall_func(16)]
pub fn sys_fremovexattr(fd: usize, name: *const u8) -> isize {
    let process = current_task().unwrap();
    let name = process.transfer_str(name);
    let file = process.get_file(fd);
    if file.is_none() {
        return -1;
    }
    let file = file.unwrap();
    let res = vfs_removexattr_by_file(file.get_file(), name.as_str());
    if res.is_err() {
        return -1;
    }
    0
}

/// 一个系统调用，用于在文件描述符之间传递数据。
///
/// 从 `in_fd` 读取最多 `count` 个字符，存到 `out_fd` 中。
/// - 如果 `offset != 0`，则其指定了 `in_fd` 中文件的偏移，此时完成后会修改 `offset` 为读取后的位置，但不更新文件内部的 `offset`
/// - 否则，正常更新文件内部的 `offset`
///
/// Reference: [send_file](https://man7.org/linux/man-pages/man2/sendfile64.2.html)
#[syscall_func(71)]
pub fn send_file(out_fd: usize, in_fd: usize, offset_ptr: usize, count: usize) -> isize {
    warn!(
        "send_file: in_fd: {:?}, out_fd: {:?}, offset_ptr: {:?}, count: {:?}",
        in_fd, out_fd, offset_ptr, count
    );
    let task = current_task().unwrap();
    let in_file = task.get_file(in_fd);
    let out_file = task.get_file(out_fd);
    if in_file.is_none() | out_file.is_none() {
        return -1;
    }
    let in_file = in_file.unwrap();
    let out_file = out_file.unwrap();
    let mut offset = if offset_ptr == 0 {
        in_file.get_file().access_inner().f_pos
    } else {
        let offset_ptr = task.transfer_raw_ptr(offset_ptr as *mut usize);
        let offset = *offset_ptr;
        warn!("send_file: offset: {:?}", offset);
        let res = vfs_llseek(in_file.get_file(), SeekFrom::Start(offset as u64)).unwrap();
        res as usize
    };
    let old_offset = offset;
    let mut read_buf = [0; 512];
    let mut read = 0;
    let mut write = 0;
    while read <= count {
        let r =
            vfs_read_file::<VfsProvider>(in_file.get_file(), &mut read_buf, offset as u64).unwrap();
        if r == 0 {
            break;
        }
        read += r;
        offset += r;
        let write_offset = out_file.get_file().access_inner().f_pos;
        let r =
            vfs_write_file::<VfsProvider>(out_file.get_file(), &read_buf[..r], write_offset as u64)
                .unwrap();
        if r == 0 {
            break;
        }
        write += r;
    }
    if offset_ptr != 0 {
        let offset_ptr = task.transfer_raw_ptr(offset_ptr as *mut usize);
        *offset_ptr = offset;
        // offset 非零则要求不更新实际文件，更新这个用户给的值
        vfs_llseek(in_file.get_file(), SeekFrom::Start(old_offset as u64)).unwrap();
    } else {
        // offset 为零则要求更新实际文件
        vfs_llseek(in_file.get_file(), SeekFrom::Current((write - read) as i64)).unwrap();
    }
    write as isize
}

/// (待实现) 一个系统调用函数，用于包把含更新文件的所有内核缓冲区(包含数据块、指针块、元数据等)都flush到磁盘上。
#[syscall_func(81)]
pub fn sync() -> isize {
    0
}

/// (待实现) 一个系统调用函数，用于把打开的文件描述符fd相关的所有缓冲元数据和数据都刷新到磁盘上。
#[syscall_func(82)]
pub fn fsync(_fd: usize) -> isize {
    0
}

/// 一个地址解析函数，通过 `fd` 所指向的一个目录文件 和 相对于该目录文件的路径或绝对路径 `path` 解析出某目标文件的绝对路径。
///
/// 当传入的`path`是一个相对地址时，那么`path`会被解析成基于文件描述符`fd`所指向的目录地址的一个地址；当传入的`path`是一个相对地址并且
/// `fd`被特殊的设置为`AT_FDCWD`时，`path`会被解析成基于调用该系统调用的进程当前工作目录的一个地址；当传入的`path`是一个绝对地址时，`fd`将被直接忽略。
///
/// 在`Alien`使用的`rvfs`中，对一个文件路径`path`是相对路径还是绝对路径的的判断条件如下：
/// + 绝对路径：以`\`开头，如`\file1.txt`，表示根目录下的`file1.txt`文件；
/// + 相对路径: 以`.\`或者`..\`或者其它开头，如`.\file1.txt`，表示`dirfd`所指向的目录下的`file1.txt`文件。
fn user_path_at(fd: isize, path: &str, flag: LookUpFlags) -> Result<String, ()> {
    let process = current_task().unwrap();
    let path = ParsePathType::from(path);
    let res = if path.is_relative() {
        if fd == AT_FDCWD {
            let fs_context = process.access_inner().fs_info.clone();
            vfs_lookup_path(fs_context.cwd, fs_context.cmnt, path, flag).map_err(|_| ())
        } else {
            let fd = fd as usize;
            let file = process.get_file(fd);
            if file.is_none() {
                return Err(());
            }
            let file = file.unwrap();
            vfs_lookup_path(file.f_dentry.clone(), file.f_mnt.clone(), path, flag).map_err(|_| ())
        }
    } else {
        Ok(path.path())
    };
    res
}
