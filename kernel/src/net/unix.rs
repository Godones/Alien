use alloc::string::String;
use alloc::sync::Arc;
use core::cell::UnsafeCell;

use syscall_define::LinuxErrno;

use crate::fs::file::KFile;

#[allow(unused)]
pub struct UnixSocket {
    file_path: UnsafeCell<Option<String>>,
    file: UnsafeCell<Option<Arc<KFile>>>,
}

impl UnixSocket {
    pub fn new() -> Self {
        Self {
            file_path: UnsafeCell::new(None),
            file: UnsafeCell::new(None),
        }
    }
    pub fn connect(&self, _file_path: String) -> Result<(), LinuxErrno> {
        //
        Err(LinuxErrno::ENOENT)
    }
}
