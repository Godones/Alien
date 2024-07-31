use alloc::{collections::BTreeMap, sync::Arc};

use constants::{
    epoll::{EpollCtlOp, EpollEvent},
    io::{OpenFlags, SeekFrom},
    AlienError, AlienResult,
};
use ksync::{Mutex, MutexGuard};
use vfscore::{dentry::VfsDentry, inode::VfsInode, utils::VfsFileStat};

use crate::kfile::File;

#[derive(Debug)]
pub struct EpollFile {
    #[allow(unused)]
    flags: OpenFlags,
    interest: Mutex<BTreeMap<usize, EpollEvent>>,
}

impl EpollFile {
    pub fn new(flags: OpenFlags) -> Self {
        EpollFile {
            flags,
            interest: Mutex::new(BTreeMap::new()),
        }
    }
    pub fn ctl(&self, op: EpollCtlOp, fd: usize, events: EpollEvent) -> AlienResult<()> {
        match op {
            EpollCtlOp::EpollCtlAdd => {
                self.interest.lock().insert(fd, events);
                Ok(())
            }
            EpollCtlOp::EpollCtlDel => {
                self.interest.lock().remove(&fd);
                Ok(())
            }
            EpollCtlOp::EpollCtlMod => {
                self.interest.lock().insert(fd, events);
                Ok(())
            }
        }
    }
    pub fn interest(&self) -> MutexGuard<BTreeMap<usize, EpollEvent>> {
        self.interest.lock()
    }
}

impl File for EpollFile {
    fn read(&self, _buf: &mut [u8]) -> AlienResult<usize> {
        todo!()
    }

    fn write(&self, _buf: &[u8]) -> AlienResult<usize> {
        todo!()
    }

    fn seek(&self, _pos: SeekFrom) -> AlienResult<u64> {
        Err(AlienError::ENOSYS)
    }

    fn get_attr(&self) -> AlienResult<VfsFileStat> {
        todo!()
    }

    fn dentry(&self) -> Arc<dyn VfsDentry> {
        panic!("EpollFile does not have dentry")
    }

    fn inode(&self) -> Arc<dyn VfsInode> {
        panic!("EpollFile does not have inode")
    }

    fn is_readable(&self) -> bool {
        true
    }

    fn is_writable(&self) -> bool {
        true
    }

    fn is_append(&self) -> bool {
        true
    }
}
