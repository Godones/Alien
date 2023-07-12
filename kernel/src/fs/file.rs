use alloc::string::String;
use alloc::sync::Arc;
use core::fmt::Debug;
use core::ops::{Deref, DerefMut};

use crate::net::socket::Socket;

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



/// kfile + socket
#[derive(Debug)]
pub enum FileLike {
    NormalFile(Arc<KFile>),
    Socket(Arc<Socket>),
}

pub enum FileType {
    NormalFile,
    Socket,
}

impl FileLike {
    pub fn get_type(&self) -> FileType {
        match self {
            FileLike::NormalFile(_) => FileType::NormalFile,
            FileLike::Socket(_) => FileType::Socket,
        }
    }

    pub fn get_nf(&self) -> Arc<KFile> {
        match self {
            FileLike::NormalFile(nf) => nf.clone(),
            FileLike::Socket(_) => panic!("get a socket file"),
        }
    }

    pub fn get_socket(&self) -> Arc<Socket> {
        match self {
            FileLike::NormalFile(_) => panic!("get a normal file when want a socket"),
            FileLike::Socket(s) => s.clone(),
        }
    }

}