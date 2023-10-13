use alloc::string::{String, ToString};
use alloc::sync::Arc;
use core::fmt::Debug;
use core::ops::{Deref, DerefMut};
use rvfs::file::OpenFlags;

use crate::ksync::{Mutex, MutexGuard};

use crate::net::socket::SocketData;
use crate::timer::TimeSpec;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum FileState {
    Valid,
    Unlink(String),
}

/// like file descriptor in linux
pub struct KFile {
    file: Arc<rvfs::file::File>,
    inner: Mutex<KFileInner>,
}

#[derive(Debug)]
pub struct KFileInner {
    pub flags: OpenFlags,
    pub state: FileState,
    pub atime: TimeSpec,
    pub mtime: TimeSpec,
    pub ctime: TimeSpec,
    pub path: String,
}

impl KFile {
    pub fn new(file: Arc<rvfs::file::File>) -> Arc<Self> {
        let flags = file.access_inner().flags;
        Arc::new(Self {
            file,
            inner: Mutex::new(KFileInner {
                flags,
                state: FileState::Valid,
                atime: TimeSpec::now(),
                mtime: TimeSpec::now(),
                ctime: TimeSpec::now(),
                path: "".to_string(),
            }),
        })
    }
    pub fn access_inner(&self) -> MutexGuard<KFileInner> {
        self.inner.lock()
    }
    pub fn is_valid(&self) -> bool {
        self.inner.lock().state == FileState::Valid
    }
    pub fn set_unlink(&self, path: String) {
        self.inner.lock().state = FileState::Unlink(path);
    }
    pub fn is_unlink(&self) -> bool {
        if let FileState::Unlink(_) = self.inner.lock().state {
            true
        } else {
            false
        }
    }
    pub fn get_file(&self) -> Arc<rvfs::file::File> {
        self.file.clone()
    }

    pub fn unlink_path(&self) -> Option<String> {
        match self.inner.lock().state {
            FileState::Unlink(ref path) => Some(path.clone()),
            _ => None,
        }
    }
    pub fn set_nonblock(&self) {
        self.get_file().access_inner().flags |= OpenFlags::O_NONBLOCK;
    }
    pub fn set_close_on_exec(&self) {
        self.inner.lock().flags |= OpenFlags::O_CLOSEEXEC;
    }
}

impl Debug for KFile {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("KFile")
            .field("file", &self.file)
            .field("state", &self.inner)
            .finish()
    }
}

impl Deref for KFile {
    type Target = Arc<rvfs::file::File>;

    fn deref(&self) -> &Self::Target {
        &self.file
    }
}

impl DerefMut for KFile {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.file
    }
}

pub trait FilePollExt {
    /// 已准备好读。对于 pipe 来说，这意味着读端的buffer内有值
    fn ready_to_read(&self) -> bool {
        true
    }
    /// 已准备好写。对于 pipe 来说，这意味着写端的buffer未满
    fn ready_to_write(&self) -> bool {
        true
    }
    /// 是否已经终止。对于 pipe 来说，这意味着另一端已关闭
    fn is_hang_up(&self) -> bool {
        false
    }
    /// 处于“意外情况”。在 (p)select 和 (p)poll 中会使用到
    #[allow(unused)]
    fn in_exceptional_conditions(&self) -> bool {
        false
    }
}

impl FilePollExt for KFile {
    /// 已准备好读。对于 pipe 来说，这意味着读端的buffer内有值
    fn ready_to_read(&self) -> bool {
        let file = self.file.clone();
        let is_ready_read = file.access_inner().f_ops_ext.is_ready_read;
        is_ready_read(file)
    }
    /// 已准备好写。对于 pipe 来说，这意味着写端的buffer未满
    fn ready_to_write(&self) -> bool {
        let file = self.file.clone();
        let is_ready_write = file.access_inner().f_ops_ext.is_ready_write;
        is_ready_write(file)
    }
    /// 是否已经终止。对于 pipe 来说，这意味着另一端已关闭
    fn is_hang_up(&self) -> bool {
        let file = self.file.clone();
        let is_hang_up = file.access_inner().f_ops_ext.is_hang_up;
        is_hang_up(file)
    }
    /// 处于“意外情况”。在 (p)select 和 (p)poll 中会使用到
    #[allow(unused)]
    fn in_exceptional_conditions(&self) -> bool {
        let file = self.file.clone();
        let is_ready_exception = file.access_inner().f_ops_ext.is_ready_exception;
        is_ready_exception(file)
    }
}

pub trait FileIoctlExt {
    fn ioctl(&self, cmd: u32, arg: usize) -> isize;
}

impl FileIoctlExt for KFile {
    fn ioctl(&self, cmd: u32, arg: usize) -> isize {
        let file = self.file.clone();
        let ioctl = file.access_inner().f_ops_ext.ioctl;
        ioctl(file, cmd, arg)
    }
}

pub trait FileSocketExt {
    fn get_socketdata_mut(&self) -> &mut SocketData;
    fn get_socketdata(&self) -> &SocketData;
}

impl FileSocketExt for KFile {
    fn get_socketdata_mut(&self) -> &mut SocketData {
        let file = self.get_file();
        let dentry_inner = file.f_dentry.access_inner();
        let inode_inner = dentry_inner.d_inode.access_inner();
        let data = inode_inner.data.as_ref().unwrap();
        SocketData::from_ptr(data.data())
    }

    fn get_socketdata(&self) -> &SocketData {
        self.get_socketdata_mut()
    }
}

mod kernel_file {
    use alloc::sync::Arc;
    use vfscore::dentry::VfsDentry;

    use crate::ksync::Mutex;
    use syscall_define::io::{OpenFlags, SeekFrom};
    use vfscore::error::VfsError;
    use vfscore::utils::FileStat;
    use vfscore::VfsResult;

    struct KernelFile {
        pos: Mutex<u64>,
        open_flag: Mutex<OpenFlags>,
        dentry: Arc<dyn VfsDentry>,
    }

    impl KernelFile {
        pub fn new(dentry: Arc<dyn VfsDentry>, open_flag: OpenFlags) -> Self {
            Self {
                pos: Mutex::new(0),
                open_flag: Mutex::new(open_flag),
                dentry,
            }
        }
    }

    pub trait File {
        fn read(&self, buf: &mut [u8]) -> VfsResult<usize>;
        fn write(&self, buf: &[u8]) -> VfsResult<usize>;
        fn read_at(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize>;
        fn write_at(&self, offset: u64, buf: &[u8]) -> VfsResult<usize>;
        fn flush(&self) -> VfsResult<()>;
        fn fsync(&self) -> VfsResult<()>;
        fn seek(&self, pos: SeekFrom) -> VfsResult<u64>;
        /// Gets the file attributes.
        fn get_attr(&self) -> VfsResult<FileStat>;
    }

    impl File for KernelFile {
        fn read(&self, buf: &mut [u8]) -> VfsResult<usize> {
            let mut pos = self.pos.lock();
            let read = self.read_at(*pos, buf)?;
            *pos += read as u64;
            Ok(read)
        }

        fn write(&self, buf: &[u8]) -> VfsResult<usize> {
            let mut pos = self.pos.lock();
            let write = self.write_at(*pos, buf)?;
            *pos += write as u64;
            Ok(write)
        }
        fn read_at(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
            let open_flag = self.open_flag.lock();
            if !open_flag.contains(OpenFlags::O_RDONLY | OpenFlags::O_RDWR) {
                return Err(VfsError::PermissionDenied);
            }
            let inode = self.dentry.inode()?;
            let read = inode.read_at(offset, buf)?;
            Ok(read)
        }
        fn write_at(&self, offset: u64, buf: &[u8]) -> VfsResult<usize> {
            let open_flag = self.open_flag.lock();
            if !open_flag.contains(OpenFlags::O_WRONLY | OpenFlags::O_RDWR) {
                return Err(VfsError::PermissionDenied);
            }
            let inode = self.dentry.inode()?;
            let write = inode.write_at(offset, buf)?;
            Ok(write)
        }
        fn flush(&self) -> VfsResult<()> {
            let open_flag = self.open_flag.lock();
            if !open_flag.contains(OpenFlags::O_WRONLY | OpenFlags::O_RDWR) {
                return Err(VfsError::PermissionDenied);
            }
            let inode = self.dentry.inode()?;
            inode.flush()
        }
        fn fsync(&self) -> VfsResult<()> {
            let open_flag = self.open_flag.lock();
            if !open_flag.contains(OpenFlags::O_WRONLY | OpenFlags::O_RDWR) {
                return Err(VfsError::PermissionDenied);
            }
            let inode = self.dentry.inode()?;
            inode.fsync()
        }
        fn seek(&self, pos: SeekFrom) -> VfsResult<u64> {
            let mut spos = self.pos.lock();
            let size = self.get_attr()?.st_size;
            let new_offset = match pos {
                SeekFrom::Start(pos) => Some(pos),
                SeekFrom::Current(off) => spos.checked_add_signed(off),
                SeekFrom::End(off) => size.checked_add_signed(off),
            }
            .ok_or_else(|| VfsError::Invalid)?;
            *spos = new_offset;
            Ok(new_offset)
        }
        /// Gets the file attributes.
        fn get_attr(&self) -> VfsResult<FileStat> {
            self.dentry.inode()?.get_attr()
        }
    }
}
