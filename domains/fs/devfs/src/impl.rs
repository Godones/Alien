use alloc::string::ToString;
use core::fmt::Debug;

use constants::AlienResult;
use generic::GenericFsDomain;
use interface::{Basic, DevFsDomain, DirEntryWrapper, DomainType, FsDomain, InodeID};
use rref::{RRef, RRefVec};
use vfscore::{fstype::FileSystemFlags, inode::InodeAttr, superblock::SuperType, utils::*};

use crate::{DEV_MAP, TASK_DOMAIN};

#[derive(Debug)]
pub struct DevFsDomainImpl {
    generic_fs: GenericFsDomain,
}

impl DevFsDomainImpl {
    pub fn new(generic_fs_domain: GenericFsDomain) -> Self {
        Self {
            generic_fs: generic_fs_domain,
        }
    }
}

impl FsDomain for DevFsDomainImpl {
    fn init(&self) -> AlienResult<()> {
        let task_domain = basic::get_domain("task").unwrap();
        match task_domain {
            DomainType::TaskDomain(task) => {
                TASK_DOMAIN.call_once(|| task);
            }
            _ => panic!("task domain not found"),
        }
        self.generic_fs.init()
    }

    fn mount(&self, mp: &RRefVec<u8>, dev_inode: Option<InodeID>) -> AlienResult<InodeID> {
        self.generic_fs.mount(mp, dev_inode)
    }

    fn drop_inode(&self, inode: InodeID) -> AlienResult<()> {
        self.generic_fs.drop_inode(inode)
    }

    fn read_at(
        &self,
        inode: InodeID,
        offset: u64,
        buf: RRefVec<u8>,
    ) -> AlienResult<(RRefVec<u8>, usize)> {
        self.generic_fs.read_at(inode, offset, buf)
    }

    fn write_at(&self, inode: InodeID, offset: u64, buf: &RRefVec<u8>) -> AlienResult<usize> {
        self.generic_fs.write_at(inode, offset, buf)
    }

    fn readdir(
        &self,
        inode: InodeID,
        start_index: usize,
        entry: RRef<DirEntryWrapper>,
    ) -> AlienResult<RRef<DirEntryWrapper>> {
        self.generic_fs.readdir(inode, start_index, entry)
    }

    fn poll(&self, inode: InodeID, mask: VfsPollEvents) -> AlienResult<VfsPollEvents> {
        self.generic_fs.poll(inode, mask)
    }

    fn ioctl(&self, inode: InodeID, cmd: u32, arg: usize) -> AlienResult<usize> {
        self.generic_fs.ioctl(inode, cmd, arg)
    }

    fn flush(&self, inode: InodeID) -> AlienResult<()> {
        self.generic_fs.flush(inode)
    }

    fn fsync(&self, inode: InodeID) -> AlienResult<()> {
        self.generic_fs.fsync(inode)
    }

    fn rmdir(&self, parent: InodeID, name: &RRefVec<u8>) -> AlienResult<()> {
        self.generic_fs.rmdir(parent, name)
    }

    fn node_permission(&self, inode: InodeID) -> AlienResult<VfsNodePerm> {
        self.generic_fs.node_permission(inode)
    }

    fn create(
        &self,
        parent: InodeID,
        name: &RRefVec<u8>,
        ty: VfsNodeType,
        perm: VfsNodePerm,
        rdev: Option<u64>,
    ) -> AlienResult<InodeID> {
        self.generic_fs.create(parent, name, ty, perm, rdev)
    }

    fn link(&self, parent: InodeID, name: &RRefVec<u8>, src: InodeID) -> AlienResult<InodeID> {
        self.generic_fs.link(parent, name, src)
    }

    fn unlink(&self, parent: InodeID, name: &RRefVec<u8>) -> AlienResult<()> {
        self.generic_fs.unlink(parent, name)
    }

    fn symlink(
        &self,
        parent: InodeID,
        name: &RRefVec<u8>,
        link: &RRefVec<u8>,
    ) -> AlienResult<InodeID> {
        self.generic_fs.symlink(parent, name, link)
    }

    fn lookup(&self, parent: InodeID, name: &RRefVec<u8>) -> AlienResult<InodeID> {
        self.generic_fs.lookup(parent, name)
    }

    fn readlink(&self, inode: InodeID, buf: RRefVec<u8>) -> AlienResult<(RRefVec<u8>, usize)> {
        self.generic_fs.readlink(inode, buf)
    }

    fn set_attr(&self, inode: InodeID, attr: InodeAttr) -> AlienResult<()> {
        self.generic_fs.set_attr(inode, attr)
    }

    fn get_attr(&self, inode: InodeID) -> AlienResult<VfsFileStat> {
        let attr = self.generic_fs.get_attr(inode);
        attr
    }

    fn inode_type(&self, inode: InodeID) -> AlienResult<VfsNodeType> {
        self.generic_fs.inode_type(inode)
    }

    fn truncate(&self, inode: InodeID, len: u64) -> AlienResult<()> {
        self.generic_fs.truncate(inode, len)
    }

    fn rename(
        &self,
        old_parent: InodeID,
        old_name: &RRefVec<u8>,
        new_parent: InodeID,
        new_name: &RRefVec<u8>,
        flags: VfsRenameFlag,
    ) -> AlienResult<()> {
        self.generic_fs
            .rename(old_parent, old_name, new_parent, new_name, flags)
    }

    fn update_time(&self, inode: InodeID, time: VfsTime, now: VfsTimeSpec) -> AlienResult<()> {
        self.generic_fs.update_time(inode, time, now)
    }

    fn sync_fs(&self, wait: bool) -> AlienResult<()> {
        self.generic_fs.sync_fs(wait)
    }

    fn stat_fs(&self, fs_stat: RRef<VfsFsStat>) -> AlienResult<RRef<VfsFsStat>> {
        self.generic_fs.stat_fs(fs_stat)
    }

    fn super_type(&self) -> AlienResult<SuperType> {
        self.generic_fs.super_type()
    }

    fn kill_sb(&self) -> AlienResult<()> {
        self.generic_fs.kill_sb()
    }

    fn fs_flag(&self) -> AlienResult<FileSystemFlags> {
        self.generic_fs.fs_flag()
    }
    fn fs_name(&self, name: RRefVec<u8>) -> AlienResult<(RRefVec<u8>, usize)> {
        self.generic_fs.fs_name(name)
    }
}

impl Basic for DevFsDomainImpl {}

impl DevFsDomain for DevFsDomainImpl {
    fn register(&self, rdev: u64, device_domain_name: &RRefVec<u8>) -> AlienResult<()> {
        let name = core::str::from_utf8(device_domain_name.as_slice()).unwrap();
        let mut map = DEV_MAP.lock();
        map.insert(rdev, name.to_string());
        Ok(())
    }
}
