use alloc::string::String;
use alloc::sync::Arc;
use core::fmt::Debug;
use core::ops::{Deref, DerefMut};

use rvfs::file::{File, OpenFlags};

use kernel_sync::{Mutex, MutexGuard};

use crate::timer::TimeSpec;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum FileState {
    Valid,
    Unlink(String),
}

/// like file descriptor in linux
pub struct KFile {
    file: Arc<File>,
    inner: Mutex<KFileInner>,
}

#[derive(Debug)]
pub struct KFileInner {
    pub flags: OpenFlags,
    pub state: FileState,
    pub atime: TimeSpec,
    pub mtime: TimeSpec,
    pub ctime: TimeSpec,
}

impl KFile {
    pub fn new(file: Arc<File>) -> Arc<Self> {
        let flags = file.access_inner().flags;
        Arc::new(Self {
            file,
            inner: Mutex::new(KFileInner {
                flags,
                state: FileState::Valid,
                atime: TimeSpec::now(),
                mtime: TimeSpec::now(),
                ctime: TimeSpec::now(),
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
    pub fn get_file(&self) -> Arc<File> {
        self.file.clone()
    }

    pub fn unlink_path(&self) -> Option<String> {
        match self.inner.lock().state {
            FileState::Unlink(ref path) => Some(path.clone()),
            _ => None,
        }
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
    type Target = Arc<File>;

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
