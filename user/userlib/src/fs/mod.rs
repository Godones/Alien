use alloc::format;
use alloc::string::{String, ToString};
use core::fmt::{Debug, Formatter, Pointer};

use bitflags::bitflags;

pub use attr::*;

use crate::syscall::*;

mod attr;

bitflags! {
    pub struct OpenFlags:u32{
        const O_RDONLY = 0x0;
        const O_WRONLY = 0x1;
        const O_RDWR = 0x2;
        const O_CREAT = 0x40;
        const O_EXCL = 0x200;
        const O_NOCTTY = 0x400;
        const O_TRUNC = 0x1000;
        const O_APPEND = 0x2000;
        const O_NONBLOCK = 0x4000;
        const O_NOFOLLOW = 0x400000;
        const O_DIRECTORY = 0x200000;
    }
}
bitflags! {
    pub struct FileMode:u32{
        const FMODE_READ = 0x0;
        const FMODE_WRITE = 0x1;
        const FMODE_RDWR = 0x2;
        const FMODE_EXEC = 0x5; //read and execute
    }
}

#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct Stat {
    pub st_dev: u64,
    pub st_ino: u64,
    pub st_mode: u32,
    pub st_nlink: u32,
    pub st_uid: u32,
    pub st_gid: u32,
    pub st_rdev: u64,
    __pad: u64,
    pub st_size: u64,
    pub st_blksize: u32,
    __pad2: u32,
    pub st_blocks: u64,
    pub st_atime_sec: u64,
    pub st_atime_nsec: u64,
    pub st_mtime_sec: u64,
    pub st_mtime_nsec: u64,
    pub st_ctime_sec: u64,
    pub st_ctime_nsec: u64,
    unused: u64,
} //128

#[derive(Default, Debug, Clone)]
#[repr(C)]
pub struct StatTime {
    pub year: u32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}
bitflags! {
    #[derive(Default)]
     pub struct InodeMode:u32{
        const S_SYMLINK = 0120000;
        const S_DIR = 0040000;
        const S_FILE = 0100000;
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Dirent64 {
    /// ino is an inode number
    pub ino: u64,
    /// off is an offset to next linux_dirent
    pub off: i64,
    /// reclen is the length of this linux_dirent
    pub reclen: u16,
    /// type is the file type
    pub type_: DirentType,
    /// name is the filename (null-terminated)
    pub name: [u8; 0],
}

impl Dirent64 {
    pub fn get_name(&self) -> &str {
        unsafe {
            let name = self.name.as_ptr();
            let name = core::ffi::CStr::from_ptr(name as *const i8);
            name.to_str().unwrap()
        }
    }
    pub fn len(&self) -> usize {
        self.reclen as usize
    }
}

bitflags! {
    pub struct DirentType:u8{
        const DT_UNKNOWN = 0;
        const DT_FIFO = 1;
        const DT_CHR = 2;
        const DT_DIR = 4;
        const DT_BLK = 6;
        const DT_REG = 8;
        const DT_LNK = 10;
        const DT_SOCK = 12;
        const DT_WHT = 14;
    }
}

impl ToString for DirentType {
    fn to_string(&self) -> String {
        match *self {
            DirentType::DT_UNKNOWN => "unknown".to_string(),
            DirentType::DT_FIFO => "fifo".to_string(),
            DirentType::DT_CHR => "char".to_string(),
            DirentType::DT_DIR => "dir".to_string(),
            DirentType::DT_BLK => "block".to_string(),
            DirentType::DT_REG => "regular".to_string(),
            DirentType::DT_LNK => "link".to_string(),
            DirentType::DT_SOCK => "sock".to_string(),
            DirentType::DT_WHT => "whiteout".to_string(),
            _ => "unknown".to_string(),
        }
    }
}

pub fn read(fd: usize, buf: &mut [u8]) -> isize {
    sys_read(fd, buf.as_mut_ptr(), buf.len())
}

pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf.as_ptr(), buf.len())
}

pub fn readdir(fd: usize, buf: &mut [u8]) -> isize {
    sys_read(fd, buf.as_mut_ptr(), buf.len())
}

pub fn list(path: &str) -> isize {
    if !path.ends_with('\0') {
        let mut p = String::from(path);
        p.push('\0');
        return sys_list(p.as_ptr());
    }
    sys_list(path.as_ptr())
}

pub fn open(name: &str, flag: OpenFlags) -> isize {
    sys_openat(
        AT_FDCWD,
        name.as_ptr(),
        flag.bits as usize,
        FileMode::FMODE_RDWR.bits() as usize,
    )
}

/// now we don't support mode
pub fn openat(fd: isize, name: &str, flag: OpenFlags, file_mode: FileMode) -> isize {
    sys_openat(
        fd,
        name.as_ptr(),
        flag.bits as usize,
        file_mode.bits() as usize,
    )
}

pub fn close(fd: usize) -> isize {
    sys_close(fd)
}

pub fn get_cwd(buf: &mut [u8]) -> Result<&str, IoError> {
    let len = sys_get_cwd(buf.as_mut_ptr(), buf.len());
    if len == -1 {
        return Err(IoError::BufferTooSmall);
    } else {
        let res = buf.iter().enumerate().find(|(_, &x)| x == 0);
        let len = if res.is_none() {
            buf.len()
        } else {
            res.unwrap().0
        };
        let s = core::str::from_utf8(&buf[..len as usize]).unwrap();
        Ok(s)
    }
}

pub fn chdir(path: &str) -> isize {
    sys_chdir(path.as_ptr())
}

pub fn mkdir(path: &str) -> isize {
    sys_mkdir(path.as_ptr())
}

pub fn seek(fd: usize, offset: isize, whence: usize) -> isize {
    sys_lseek(fd, offset, whence)
}

pub fn fstat(fd: usize, stat: &mut Stat) -> isize {
    sys_fstat(fd, stat as *mut Stat as *mut u8)
}

pub fn linkat(
    old_fd: isize,
    old_path: &str,
    new_fd: usize,
    new_path: &str,
    flag: LinkFlags,
) -> isize {
    sys_linkat(
        old_fd,
        old_path.as_ptr(),
        new_fd,
        new_path.as_ptr(),
        flag.bits() as usize,
    )
}

pub fn unlinkat(fd: isize, path: &str, flag: usize) -> isize {
    sys_unlinkat(fd, path.as_ptr(), flag)
}

pub fn symlinkat(old_path: &str, new_fd: isize, new_path: &str) -> isize {
    sys_symlinkat(old_path.as_ptr(), new_fd, new_path.as_ptr())
}

pub fn readlinkat(fd: isize, path: &str, buf: &mut [u8]) -> isize {
    sys_readlinkat(fd, path.as_ptr(), buf.as_mut_ptr(), buf.len())
}

pub fn fstatat(fd: isize, path: &str, stat: &mut Stat, flag: StatFlags) -> isize {
    sys_fstatat(
        fd,
        path.as_ptr(),
        stat as *mut Stat as *mut u8,
        flag.bits() as usize,
    )
}

pub fn statfs(path: &str, stat: &mut StatFs) -> isize {
    sys_statfs(path.as_ptr(), stat as *mut StatFs as *mut u8)
}

pub fn fstatfs(fd: usize, stat: &mut StatFs) -> isize {
    sys_fstatfs(fd, stat as *mut StatFs as *mut u8)
}

pub fn renameat(old_fd: isize, old_path: &str, new_fd: isize, new_path: &str) -> isize {
    sys_renameat(old_fd, old_path.as_ptr(), new_fd, new_path.as_ptr())
}

pub fn mkdirat(fd: isize, path: &str, flag: OpenFlags) -> isize {
    sys_mkdirat(fd, path.as_ptr(), flag.bits as usize)
}

#[derive(Debug)]
pub enum IoError {
    BufferTooSmall,
    FileNotFound,
    FileAlreadyExist,
}
bitflags! {
    pub struct LinkFlags:u32{
        /// Follow symbolic links.
        const AT_SYMLINK_FOLLOW = 0x400;
        /// Allow empty relative pathname.
        const AT_EMPTY_PATH = 0x1000;
    }
}

bitflags! {
    pub struct StatFlags:u32{
        const AT_EMPTY_PATH = 0x1000;
        const AT_NO_AUTOMOUNT = 0x800;
        const AT_SYMLINK_NOFOLLOW = 0x100;
    }
}

#[derive(Default)]
#[repr(C)]
pub struct StatFs {
    pub fs_type: u32,
    pub block_size: u64,
    pub total_blocks: u64,
    pub free_blocks: u64,
    pub total_inodes: u64,
    pub name_len: u32,
    pub name: [u8; 32],
}

impl Debug for StatFs {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("StatFs")
            .field("fs_type", &self.fs_type)
            .field("block_size", &self.block_size)
            .field("total_blocks", &self.total_blocks)
            .field("free_blocks", &self.free_blocks)
            .field("total_inodes", &self.total_inodes)
            .field("name_len", &self.name_len)
            .field("name", &core::str::from_utf8(&self.name).unwrap())
            .finish()
    }
}

pub const AT_FDCWD: isize = -100isize;
