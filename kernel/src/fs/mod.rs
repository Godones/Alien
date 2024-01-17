pub mod basic;
pub mod control;
pub mod ext;
pub mod link;
pub mod poll;
pub mod select;
pub mod stdio;

use crate::task::{current_task, FsContext};
use alloc::vec::Vec;
use constants::io::InodeMode;
use constants::{AlienResult, LinuxErrno, AT_FDCWD};
use log::info;
use vfscore::path::{SysContext, VfsPath};
use vfscore::utils::{VfsInodeMode, VfsNodeType};

/// 地址解析函数，通过 `fd` 所指向的一个目录文件 和 相对于该目录文件的路径或绝对路径 `path` 解析出某目标文件的绝对路径。
///
/// 当传入的`path`是一个相对地址时，那么`path`会被解析成基于文件描述符`fd`所指向的目录地址的一个地址；当传入的`path`是一个相对地址并且
/// `fd`被特殊的设置为`AT_FDCWD`时，`path`会被解析成基于调用该系统调用的进程当前工作目录的一个地址；当传入的`path`是一个绝对地址时，`fd`将被直接忽略。
///
/// 在`Alien`使用的`rvfs`中，对一个文件路径`path`是相对路径还是绝对路径的的判断条件如下：
/// + 绝对路径：以`/`开头，如`/file1.txt`，表示根目录下的`file1.txt`文件；
/// + 相对路径: 以`./`或者`../`或者其它开头，如`./file1.txt`，表示`dirfd`所指向的目录下的`file1.txt`文件。
fn user_path_at(fd: isize, path: &str) -> AlienResult<VfsPath> {
    info!("user_path_at fd: {},path:{}", fd, path);
    let process = current_task().unwrap();
    let res = if !path.starts_with("/") {
        if fd == AT_FDCWD {
            let fs_context = process.access_inner().fs_info.clone();
            VfsPath::new(fs_context.cwd).join(path)
        } else {
            let fd = fd as usize;
            let file = process.get_file(fd).ok_or(LinuxErrno::EBADF)?;
            VfsPath::new(file.dentry()).join(path)
        }
    } else {
        VfsPath::new(vfs::system_root_fs()).join(path)
    };
    res.map_err(|e| e.into())
}

pub fn read_all(file_name: &str, buf: &mut Vec<u8>) -> bool {
    let task = current_task();
    // let cwd = if task.is_some() {
    //     task.unwrap().access_inner().cwd().cwd
    // } else {
    //     vfs::system_root_fs()
    // };
    let path = if task.is_none() {
        VfsPath::new(vfs::system_root_fs()).join(file_name).unwrap()
    } else {
        user_path_at(AT_FDCWD, file_name).unwrap()
    };

    let dentry = path.open(None);
    if dentry.is_err() {
        info!("open file {} failed, err:{:?}", file_name, dentry.err());
        return false;
    }
    let dentry = dentry.unwrap();
    if dentry.inode().unwrap().inode_type() != VfsNodeType::File {
        info!("{} is not a file", file_name);
        return false;
    }
    let size = dentry.inode().unwrap().get_attr().unwrap().st_size;
    let mut offset = 0;
    while offset < size {
        let mut tmp = [0; 512];
        let res = dentry.inode().unwrap().read_at(offset, &mut tmp).unwrap();
        offset += res as u64;
        buf.extend_from_slice(&tmp);
    }
    assert_eq!(offset, size);
    true
}

/// [InodeMode](InodeMode)转换为[VfsInodeMode](VfsInodeMode)
fn im2vim(mode: InodeMode) -> VfsInodeMode {
    VfsInodeMode::from_bits_truncate(mode.bits())
}

fn syscontext_for_vfs(fs_info: FsContext) -> SysContext {
    SysContext {
        pid: 0,
        uid: 0,
        gid: 0,
        cwd: fs_info.cwd.clone(),
        root: fs_info.root.clone(),
    }
}
