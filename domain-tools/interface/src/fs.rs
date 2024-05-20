use downcast_rs::{impl_downcast, DowncastSync};
use gproxy::proxy;
use rref::{RRef, RRefVec};
use vfscore::{fstype::FileSystemFlags, inode::InodeAttr, superblock::SuperType, utils::*};

use super::AlienResult;
use crate::{Basic, DirEntryWrapper, InodeID};

#[proxy(FsDomainProxy)]
pub trait FsDomain: Basic + DowncastSync {
    fn init(&self) -> AlienResult<()>;
    fn mount(&self, mp: &RRefVec<u8>, dev_inode: Option<InodeID>) -> AlienResult<InodeID>;
    fn drop_inode(&self, inode: InodeID) -> AlienResult<()>;
    // file operations
    fn read_at(
        &self,
        inode: InodeID,
        offset: u64,
        buf: RRefVec<u8>,
    ) -> AlienResult<(RRefVec<u8>, usize)>;
    fn write_at(&self, inode: InodeID, offset: u64, buf: &RRefVec<u8>) -> AlienResult<usize>;
    fn readdir(
        &self,
        inode: InodeID,
        start_index: usize,
        entry: RRef<DirEntryWrapper>,
    ) -> AlienResult<RRef<DirEntryWrapper>>;
    fn poll(&self, inode: InodeID, mask: VfsPollEvents) -> AlienResult<VfsPollEvents>;
    fn ioctl(&self, inode: InodeID, cmd: u32, arg: usize) -> AlienResult<usize>;
    fn flush(&self, inode: InodeID) -> AlienResult<()>;
    fn fsync(&self, inode: InodeID) -> AlienResult<()>;

    // inode operations
    fn rmdir(&self, parent: InodeID, name: &RRefVec<u8>) -> AlienResult<()>;
    fn node_permission(&self, inode: InodeID) -> AlienResult<VfsNodePerm>;
    fn create(
        &self,
        parent: InodeID,
        name: &RRefVec<u8>,
        ty: VfsNodeType,
        perm: VfsNodePerm,
        rdev: Option<u64>,
    ) -> AlienResult<InodeID>;
    fn link(&self, parent: InodeID, name: &RRefVec<u8>, src: InodeID) -> AlienResult<InodeID>;
    fn unlink(&self, parent: InodeID, name: &RRefVec<u8>) -> AlienResult<()>;
    fn symlink(
        &self,
        parent: InodeID,
        name: &RRefVec<u8>,
        link: &RRefVec<u8>,
    ) -> AlienResult<InodeID>;
    fn lookup(&self, parent: InodeID, name: &RRefVec<u8>) -> AlienResult<InodeID>;
    fn readlink(&self, inode: InodeID, buf: RRefVec<u8>) -> AlienResult<(RRefVec<u8>, usize)>;
    fn set_attr(&self, inode: InodeID, attr: InodeAttr) -> AlienResult<()>;
    fn get_attr(&self, inode: InodeID) -> AlienResult<VfsFileStat>;
    fn inode_type(&self, inode: InodeID) -> AlienResult<VfsNodeType>;
    fn truncate(&self, inode: InodeID, len: u64) -> AlienResult<()>;
    fn rename(
        &self,
        old_parent: InodeID,
        old_name: &RRefVec<u8>,
        new_parent: InodeID,
        new_name: &RRefVec<u8>,
        flags: VfsRenameFlag,
    ) -> AlienResult<()>;
    fn update_time(&self, inode: InodeID, time: VfsTime, now: VfsTimeSpec) -> AlienResult<()>;

    // superblock operations
    fn sync_fs(&self, wait: bool) -> AlienResult<()>;
    fn stat_fs(&self, fs_stat: RRef<VfsFsStat>) -> AlienResult<RRef<VfsFsStat>>;
    fn super_type(&self) -> AlienResult<SuperType>;

    // fs
    fn kill_sb(&self) -> AlienResult<()>;
    fn fs_flag(&self) -> AlienResult<FileSystemFlags>;
    fn fs_name(&self, name: RRefVec<u8>) -> AlienResult<(RRefVec<u8>, usize)>;
}

impl_downcast!(sync FsDomain);

#[proxy(DevFsDomainProxy)]
pub trait DevFsDomain: FsDomain + DowncastSync {
    fn register(&self, rdev: u64, device_domain_name: &RRefVec<u8>) -> AlienResult<()>;
}

impl_downcast!(sync DevFsDomain);
