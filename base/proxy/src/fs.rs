use alloc::boxed::Box;

use constants::AlienResult;
use interface::{Basic, DirEntryWrapper, FsDomain, InodeID};
use rref::{RRef, RRefVec};
use vfscore::{
    fstype::FileSystemFlags,
    inode::InodeAttr,
    superblock::SuperType,
    utils::{
        VfsFileStat, VfsFsStat, VfsNodePerm, VfsNodeType, VfsPollEvents, VfsRenameFlag, VfsTime,
        VfsTimeSpec,
    },
};

#[derive(Debug)]
pub struct FsDomainProxy {
    domain_id: u64,
    domain: Box<dyn FsDomain>,
}

impl FsDomainProxy {
    pub fn new(domain_id: u64, domain: Box<dyn FsDomain>) -> Self {
        Self { domain_id, domain }
    }
}

impl Basic for FsDomainProxy {
    fn is_active(&self) -> bool {
        self.domain.is_active()
    }
}

impl FsDomain for FsDomainProxy {
    fn init(&self) -> AlienResult<()> {
        self.domain.init()
    }

    fn mount(&self, mp: &RRefVec<u8>, dev_inode: Option<InodeID>) -> AlienResult<InodeID> {
        if self.domain.is_active() {
            self.domain.mount(mp, dev_inode)
        } else {
            Err(constants::AlienError::DOMAINCRASH)
        }
    }

    fn drop_inode(&self, inode: InodeID) -> AlienResult<()> {
        if self.domain.is_active() {
            self.domain.drop_inode(inode)
        } else {
            Err(constants::AlienError::DOMAINCRASH)
        }
    }

    fn read_at(
        &self,
        inode: InodeID,
        offset: u64,
        buf: RRefVec<u8>,
    ) -> AlienResult<(RRefVec<u8>, usize)> {
        if self.domain.is_active() {
            self.domain.read_at(inode, offset, buf)
        } else {
            Err(constants::AlienError::DOMAINCRASH)
        }
    }

    fn write_at(&self, inode: InodeID, offset: u64, buf: &RRefVec<u8>) -> AlienResult<usize> {
        if self.domain.is_active() {
            self.domain.write_at(inode, offset, buf)
        } else {
            Err(constants::AlienError::DOMAINCRASH)
        }
    }

    fn readdir(
        &self,
        inode: InodeID,
        start_index: usize,
        entry: RRef<DirEntryWrapper>,
    ) -> AlienResult<RRef<DirEntryWrapper>> {
        if self.domain.is_active() {
            self.domain.readdir(inode, start_index, entry)
        } else {
            Err(constants::AlienError::DOMAINCRASH)
        }
    }

    fn poll(&self, inode: InodeID, mask: VfsPollEvents) -> AlienResult<VfsPollEvents> {
        if self.domain.is_active() {
            self.domain.poll(inode, mask)
        } else {
            Err(constants::AlienError::DOMAINCRASH)
        }
    }

    fn flush(&self, inode: InodeID) -> AlienResult<()> {
        if self.domain.is_active() {
            self.domain.flush(inode)
        } else {
            Err(constants::AlienError::DOMAINCRASH)
        }
    }

    fn fsync(&self, inode: InodeID) -> AlienResult<()> {
        if self.domain.is_active() {
            self.domain.fsync(inode)
        } else {
            Err(constants::AlienError::DOMAINCRASH)
        }
    }

    fn rmdir(&self, parent: InodeID, name: &RRefVec<u8>) -> AlienResult<()> {
        if self.domain.is_active() {
            self.domain.rmdir(parent, name)
        } else {
            Err(constants::AlienError::DOMAINCRASH)
        }
    }

    fn node_permission(&self, inode: InodeID) -> AlienResult<VfsNodePerm> {
        if self.domain.is_active() {
            self.domain.node_permission(inode)
        } else {
            Err(constants::AlienError::DOMAINCRASH)
        }
    }

    fn create(
        &self,
        parent: InodeID,
        name: &RRefVec<u8>,
        ty: VfsNodeType,
        perm: VfsNodePerm,
        rdev: Option<u64>,
    ) -> AlienResult<InodeID> {
        if self.domain.is_active() {
            self.domain.create(parent, name, ty, perm, rdev)
        } else {
            Err(constants::AlienError::DOMAINCRASH)
        }
    }

    fn link(&self, parent: InodeID, name: &RRefVec<u8>, src: InodeID) -> AlienResult<InodeID> {
        if self.domain.is_active() {
            self.domain.link(parent, name, src)
        } else {
            Err(constants::AlienError::DOMAINCRASH)
        }
    }

    fn unlink(&self, parent: InodeID, name: &RRefVec<u8>) -> AlienResult<()> {
        if self.domain.is_active() {
            self.domain.unlink(parent, name)
        } else {
            Err(constants::AlienError::DOMAINCRASH)
        }
    }

    fn symlink(
        &self,
        parent: InodeID,
        name: &RRefVec<u8>,
        link: &RRefVec<u8>,
    ) -> AlienResult<InodeID> {
        if self.domain.is_active() {
            self.domain.symlink(parent, name, link)
        } else {
            Err(constants::AlienError::DOMAINCRASH)
        }
    }

    fn lookup(&self, parent: InodeID, name: &RRefVec<u8>) -> AlienResult<InodeID> {
        if self.domain.is_active() {
            self.domain.lookup(parent, name)
        } else {
            Err(constants::AlienError::DOMAINCRASH)
        }
    }

    fn readlink(&self, inode: InodeID, buf: RRefVec<u8>) -> AlienResult<(RRefVec<u8>, usize)> {
        if self.domain.is_active() {
            self.domain.readlink(inode, buf)
        } else {
            Err(constants::AlienError::DOMAINCRASH)
        }
    }

    fn set_attr(&self, inode: InodeID, attr: InodeAttr) -> AlienResult<()> {
        if self.domain.is_active() {
            self.domain.set_attr(inode, attr)
        } else {
            Err(constants::AlienError::DOMAINCRASH)
        }
    }

    fn get_attr(&self, inode: InodeID) -> AlienResult<VfsFileStat> {
        if self.domain.is_active() {
            self.domain.get_attr(inode)
        } else {
            Err(constants::AlienError::DOMAINCRASH)
        }
    }

    fn inode_type(&self, inode: InodeID) -> AlienResult<VfsNodeType> {
        if self.domain.is_active() {
            self.domain.inode_type(inode)
        } else {
            Err(constants::AlienError::DOMAINCRASH)
        }
    }

    fn truncate(&self, inode: InodeID, len: u64) -> AlienResult<()> {
        if self.domain.is_active() {
            self.domain.truncate(inode, len)
        } else {
            Err(constants::AlienError::DOMAINCRASH)
        }
    }

    fn rename(
        &self,
        old_parent: InodeID,
        old_name: &RRefVec<u8>,
        new_parent: InodeID,
        new_name: &RRefVec<u8>,
        flags: VfsRenameFlag,
    ) -> AlienResult<()> {
        if self.domain.is_active() {
            self.domain
                .rename(old_parent, old_name, new_parent, new_name, flags)
        } else {
            Err(constants::AlienError::DOMAINCRASH)
        }
    }

    fn update_time(&self, inode: InodeID, time: VfsTime, now: VfsTimeSpec) -> AlienResult<()> {
        if self.domain.is_active() {
            self.domain.update_time(inode, time, now)
        } else {
            Err(constants::AlienError::DOMAINCRASH)
        }
    }

    fn sync_fs(&self, wait: bool) -> AlienResult<()> {
        if self.domain.is_active() {
            self.domain.sync_fs(wait)
        } else {
            Err(constants::AlienError::DOMAINCRASH)
        }
    }

    fn stat_fs(&self, fs_stat: RRef<VfsFsStat>) -> AlienResult<RRef<VfsFsStat>> {
        if self.domain.is_active() {
            self.domain.stat_fs(fs_stat)
        } else {
            Err(constants::AlienError::DOMAINCRASH)
        }
    }

    fn super_type(&self) -> AlienResult<SuperType> {
        if self.domain.is_active() {
            self.domain.super_type()
        } else {
            Err(constants::AlienError::DOMAINCRASH)
        }
    }

    fn kill_sb(&self) -> AlienResult<()> {
        if self.domain.is_active() {
            self.domain.kill_sb()
        } else {
            Err(constants::AlienError::DOMAINCRASH)
        }
    }

    fn fs_flag(&self) -> AlienResult<FileSystemFlags> {
        if self.domain.is_active() {
            self.domain.fs_flag()
        } else {
            Err(constants::AlienError::DOMAINCRASH)
        }
    }

    fn fs_name(&self, name: RRefVec<u8>) -> AlienResult<(RRefVec<u8>, usize)> {
        if self.domain.is_active() {
            self.domain.fs_name(name)
        } else {
            Err(constants::AlienError::DOMAINCRASH)
        }
    }
}
