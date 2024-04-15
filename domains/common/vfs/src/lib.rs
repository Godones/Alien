#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::{boxed::Box, collections::BTreeMap, sync::Arc};
use core::sync::atomic::AtomicU64;

use constants::{
    io::{OpenFlags, PollEvents},
    AlienError, AlienResult,
};
use interface::{Basic, DomainType, InodeID, TaskDomain, VfsDomain};
use ksync::RwLock;
use log::debug;
use rref::{RRef, RRefVec};
use spin::Once;
use vfscore::{
    dentry::VfsDentry,
    path::VfsPath,
    utils::{VfsFileStat, VfsInodeMode, VfsNodeType, VfsPollEvents},
};

use crate::{
    kfile::{File, KernelFile},
    tree::system_root_fs,
};

mod devfs;
mod initrd;
mod kfile;
mod pipefs;
mod procfs;
mod ramfs;
mod shim;
mod sys;
mod tree;

static TASK_DOMAIN: Once<Arc<dyn TaskDomain>> = Once::new();

static VFS_MAP: RwLock<BTreeMap<InodeID, Arc<dyn File>>> = RwLock::new(BTreeMap::new());
static INODE_ID: AtomicU64 = AtomicU64::new(4);

fn insert_dentry(dentry: Arc<dyn VfsDentry>, open_flags: OpenFlags) -> InodeID {
    let id = INODE_ID.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
    let file = KernelFile::new(dentry, open_flags);
    VFS_MAP.write().insert(id, Arc::new(file));
    id
}

fn remove_file(inode: InodeID) {
    VFS_MAP.write().remove(&inode);
}

fn get_file(inode: InodeID) -> Option<Arc<dyn File>> {
    VFS_MAP.read().get(&inode).map(|f| f.clone())
}

#[derive(Debug)]
struct VfsDomainImpl;

impl Basic for VfsDomainImpl {}

impl VfsDomain for VfsDomainImpl {
    fn init(&self, initrd: &[u8]) -> AlienResult<()> {
        tree::init_filesystem(initrd).unwrap();
        let task_domain = basic::get_domain("task").unwrap();
        match task_domain {
            DomainType::TaskDomain(task) => TASK_DOMAIN.call_once(|| task),
            _ => panic!("task domain not found"),
        };
        Ok(())
    }

    fn vfs_poll(&self, inode: InodeID, events: VfsPollEvents) -> AlienResult<VfsPollEvents> {
        let file = get_file(inode).unwrap();
        let res = file.poll(PollEvents::from_bits_truncate(events.bits()))?;
        Ok(VfsPollEvents::from_bits_truncate(res.bits()))
    }

    fn vfs_ioctl(&self, inode: InodeID, cmd: u32, arg: usize) -> AlienResult<usize> {
        let file = get_file(inode).unwrap();
        let res = file.ioctl(cmd, arg)?;
        Ok(res)
    }

    fn vfs_open(
        &self,
        root: InodeID,
        path: &RRefVec<u8>,
        mode: u32,
        open_flags: usize,
    ) -> AlienResult<InodeID> {
        let start = get_file(root).ok_or(AlienError::EINVAL)?;
        let root = system_root_fs();
        let path = core::str::from_utf8(path.as_slice()).unwrap();
        let open_flags = OpenFlags::from_bits_truncate(open_flags);
        let mode = if open_flags.contains(OpenFlags::O_CREAT) {
            None
        } else {
            Some(VfsInodeMode::from_bits_truncate(mode))
        };
        debug!("vfs_open:  path: {:?}, mode: {:?}", path, mode);

        let path = VfsPath::new(root, start.dentry()).join(path)?.open(mode)?;
        let id = insert_dentry(path, open_flags);
        Ok(id)
    }
    fn vfs_close(&self, inode: InodeID) -> AlienResult<()> {
        remove_file(inode);
        Ok(())
    }

    fn vfs_getattr(
        &self,
        inode: InodeID,
        mut attr: RRef<VfsFileStat>,
    ) -> AlienResult<RRef<VfsFileStat>> {
        let dentry = get_file(inode).unwrap().dentry();
        let vfs_attr = dentry.inode()?.get_attr()?;
        *attr = vfs_attr;
        Ok(attr)
    }
    fn vfs_read_at(
        &self,
        inode: InodeID,
        offset: u64,
        mut buf: RRefVec<u8>,
    ) -> AlienResult<(RRefVec<u8>, usize)> {
        let file = get_file(inode).unwrap();
        let res = file.read_at(offset, buf.as_mut_slice())?;
        Ok((buf, res))
    }
    fn vfs_read(&self, inode: InodeID, mut buf: RRefVec<u8>) -> AlienResult<(RRefVec<u8>, usize)> {
        let file = get_file(inode).unwrap();
        let res = file.read(buf.as_mut_slice())?;
        Ok((buf, res))
    }
    fn vfs_write_at(
        &self,
        inode: InodeID,
        offset: u64,
        buf: RRefVec<u8>,
    ) -> AlienResult<(RRefVec<u8>, usize)> {
        let file = get_file(inode).unwrap();
        let res = file.write_at(offset, buf.as_slice())?;
        Ok((buf, res))
    }
    fn vfs_write(&self, inode: InodeID, buf: &RRefVec<u8>) -> AlienResult<usize> {
        let file = get_file(inode).unwrap();
        let res = file.write(buf.as_slice())?;
        Ok(res)
    }
    fn vfs_flush(&self, inode: InodeID) -> AlienResult<()> {
        let file = get_file(inode).unwrap();
        file.flush()?;
        Ok(())
    }
    fn vfs_fsync(&self, inode: InodeID) -> AlienResult<()> {
        let file = get_file(inode).unwrap();
        file.fsync()?;
        Ok(())
    }
    fn vfs_inode_type(&self, inode: InodeID) -> AlienResult<VfsNodeType> {
        let file = get_file(inode).unwrap();
        let res = file.inode().inode_type();
        Ok(res)
    }
}

pub fn main() -> Box<dyn VfsDomain> {
    Box::new(VfsDomainImpl)
}
