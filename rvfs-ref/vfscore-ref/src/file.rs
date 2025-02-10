use downcast_rs::{impl_downcast, DowncastSync};
use shared_heap::DVec;

use crate::{
    error::VfsError,
    utils::{VfsDirEntry, VfsPollEvents},
    VfsResult,
};

pub trait VfsFile: Send + Sync + DowncastSync {
    fn read_at(&self, _offset: u64, _buf: DVec<u8>) -> VfsResult<(DVec<u8>, usize)> {
        Err(VfsError::NoSys)
    }
    fn write_at(&self, _offset: u64, _buf: &DVec<u8>) -> VfsResult<usize> {
        Err(VfsError::NoSys)
    }
    /// Read directory entries. This is called by the getdents(2) system call.
    ///
    /// For every call, this function will return an valid entry, or an error. If
    /// it read to the end of directory, it will return an empty entry.
    fn readdir(&self, _start_index: usize) -> VfsResult<Option<VfsDirEntry>> {
        Err(VfsError::NoSys)
    }
    fn poll(&self, event: VfsPollEvents) -> VfsResult<VfsPollEvents> {
        let mut res = VfsPollEvents::empty();
        if event.contains(VfsPollEvents::IN) {
            res |= VfsPollEvents::IN;
        }
        if event.contains(VfsPollEvents::OUT) {
            res |= VfsPollEvents::OUT;
        }
        Ok(res)
    }
    fn ioctl(&self, _cmd: u32, _arg: usize) -> VfsResult<usize> {
        Err(VfsError::NoSys)
    }
    /// Called by the close(2) system call to flush a file
    fn flush(&self) -> VfsResult<()> {
        Ok(())
    }
    /// Called by the fsync(2) system call.
    fn fsync(&self) -> VfsResult<()> {
        Ok(())
    }
}

impl_downcast!(sync VfsFile);
