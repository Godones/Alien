use alloc::boxed::Box;

use constants::{AlienError, AlienResult};
use interface::{Basic, InodeID, VfsDomain};
use rref::{RRef, RRefVec};
use vfscore::utils::{VfsFileStat, VfsNodeType, VfsPollEvents};

#[derive(Debug)]
pub struct VfsDomainProxy {
    domain: Box<dyn VfsDomain>,
}

impl VfsDomainProxy {
    pub fn new(_id: u64, domain: Box<dyn VfsDomain>) -> Self {
        Self { domain }
    }
}

impl Basic for VfsDomainProxy {
    fn is_active(&self) -> bool {
        self.domain.is_active()
    }
}

impl VfsDomain for VfsDomainProxy {
    fn init(&self) -> AlienResult<()> {
        self.domain.init()
    }

    fn vfs_poll(&self, inode: InodeID, events: VfsPollEvents) -> AlienResult<VfsPollEvents> {
        if self.domain.is_active() {
            self.domain.vfs_poll(inode, events)
        } else {
            Err(AlienError::DOMAINCRASH)
        }
    }

    fn vfs_ioctl(&self, inode: InodeID, cmd: u32, arg: usize) -> AlienResult<usize> {
        if self.domain.is_active() {
            self.domain.vfs_ioctl(inode, cmd, arg)
        } else {
            Err(AlienError::DOMAINCRASH)
        }
    }

    fn vfs_open(
        &self,
        root: InodeID,
        path: &RRefVec<u8>,
        mode: u32,
        open_flags: usize,
    ) -> AlienResult<InodeID> {
        if self.domain.is_active() {
            self.domain.vfs_open(root, path, mode, open_flags)
        } else {
            Err(AlienError::DOMAINCRASH)
        }
    }

    fn vfs_close(&self, inode: InodeID) -> AlienResult<()> {
        if self.domain.is_active() {
            self.domain.vfs_close(inode)
        } else {
            Err(AlienError::DOMAINCRASH)
        }
    }

    fn vfs_getattr(
        &self,
        inode: InodeID,
        attr: RRef<VfsFileStat>,
    ) -> AlienResult<RRef<VfsFileStat>> {
        if self.domain.is_active() {
            self.domain.vfs_getattr(inode, attr)
        } else {
            Err(AlienError::DOMAINCRASH)
        }
    }

    fn vfs_read_at(
        &self,
        inode: InodeID,
        offset: u64,
        buf: RRefVec<u8>,
    ) -> AlienResult<(RRefVec<u8>, usize)> {
        if self.domain.is_active() {
            self.domain.vfs_read_at(inode, offset, buf)
        } else {
            Err(AlienError::DOMAINCRASH)
        }
    }
    fn vfs_read(&self, inode: InodeID, buf: RRefVec<u8>) -> AlienResult<(RRefVec<u8>, usize)> {
        if self.domain.is_active() {
            self.domain.vfs_read(inode, buf)
        } else {
            Err(AlienError::DOMAINCRASH)
        }
    }
    fn vfs_write_at(
        &self,
        inode: InodeID,
        offset: u64,
        buf: RRefVec<u8>,
    ) -> AlienResult<(RRefVec<u8>, usize)> {
        if self.domain.is_active() {
            self.domain.vfs_write_at(inode, offset, buf)
        } else {
            Err(AlienError::DOMAINCRASH)
        }
    }
    fn vfs_write(&self, inode: InodeID, buf: &RRefVec<u8>) -> AlienResult<usize> {
        if self.domain.is_active() {
            self.domain.vfs_write(inode, buf)
        } else {
            Err(AlienError::DOMAINCRASH)
        }
    }

    fn vfs_flush(&self, inode: InodeID) -> AlienResult<()> {
        if self.domain.is_active() {
            self.domain.vfs_flush(inode)
        } else {
            Err(AlienError::DOMAINCRASH)
        }
    }

    fn vfs_fsync(&self, inode: InodeID) -> AlienResult<()> {
        if self.domain.is_active() {
            self.domain.vfs_fsync(inode)
        } else {
            Err(AlienError::DOMAINCRASH)
        }
    }

    fn vfs_inode_type(&self, inode: InodeID) -> AlienResult<VfsNodeType> {
        if self.domain.is_active() {
            self.domain.vfs_inode_type(inode)
        } else {
            Err(AlienError::DOMAINCRASH)
        }
    }
}